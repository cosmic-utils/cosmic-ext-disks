#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiskSegmentKind {
    Partition,
    FreeSpace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartitionExtent {
    pub id: usize,
    pub offset: u64,
    pub size: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiskSegment {
    pub kind: DiskSegmentKind,
    pub offset: u64,
    pub size: u64,
    pub partition_id: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SegmentAnomaly {
    PartitionOverlapsPrevious {
        id: usize,
        partition_offset: u64,
        previous_end: u64,
    },
    PartitionStartsPastDisk {
        id: usize,
        partition_offset: u64,
        disk_size: u64,
    },
    PartitionEndPastDisk {
        id: usize,
        partition_end: u64,
        disk_size: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SegmentComputation {
    pub segments: Vec<DiskSegment>,
    pub anomalies: Vec<SegmentAnomaly>,
}

impl DiskSegment {
    pub fn free_space(offset: u64, size: u64) -> Self {
        Self {
            kind: DiskSegmentKind::FreeSpace,
            offset,
            size,
            partition_id: None,
        }
    }

    pub fn partition(id: usize, offset: u64, size: u64) -> Self {
        Self {
            kind: DiskSegmentKind::Partition,
            offset,
            size,
            partition_id: Some(id),
        }
    }
}

pub fn compute_disk_segments(
    disk_size: u64,
    mut partitions: Vec<PartitionExtent>,
) -> SegmentComputation {
    let mut anomalies = Vec::new();

    if partitions.is_empty() {
        return SegmentComputation {
            segments: vec![DiskSegment::free_space(0, disk_size)],
            anomalies,
        };
    }

    partitions.sort_by(|a, b| a.offset.cmp(&b.offset));

    let mut segments = Vec::new();
    let mut current_offset = 0u64;

    for partition in partitions {
        if partition.size == 0 {
            continue;
        }

        if partition.offset >= disk_size {
            anomalies.push(SegmentAnomaly::PartitionStartsPastDisk {
                id: partition.id,
                partition_offset: partition.offset,
                disk_size,
            });

            if current_offset < disk_size {
                segments.push(DiskSegment::free_space(
                    current_offset,
                    disk_size.saturating_sub(current_offset),
                ));
            }
            break;
        }

        if partition.offset > current_offset {
            segments.push(DiskSegment::free_space(
                current_offset,
                partition.offset - current_offset,
            ));
            current_offset = partition.offset;
        } else if partition.offset < current_offset {
            anomalies.push(SegmentAnomaly::PartitionOverlapsPrevious {
                id: partition.id,
                partition_offset: partition.offset,
                previous_end: current_offset,
            });
        }

        let partition_end = partition.offset.saturating_add(partition.size);
        let effective_end = if partition_end > disk_size {
            anomalies.push(SegmentAnomaly::PartitionEndPastDisk {
                id: partition.id,
                partition_end,
                disk_size,
            });
            disk_size
        } else {
            partition_end
        };

        // If the partition overlaps, clamp its visible start to keep the output ordered and
        // non-overlapping for UI rendering.
        let effective_offset = current_offset.max(partition.offset);
        let effective_size = effective_end.saturating_sub(effective_offset);

        if effective_size > 0 {
            segments.push(DiskSegment::partition(
                partition.id,
                effective_offset,
                effective_size,
            ));
            current_offset = effective_offset.saturating_add(effective_size);
        }

        if current_offset >= disk_size {
            break;
        }
    }

    if current_offset < disk_size {
        segments.push(DiskSegment::free_space(
            current_offset,
            disk_size.saturating_sub(current_offset),
        ));
    }

    SegmentComputation {
        segments,
        anomalies,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(id: usize, offset: u64, size: u64) -> PartitionExtent {
        PartitionExtent { id, offset, size }
    }

    #[test]
    fn empty_partitions_is_single_free_space() {
        let res = compute_disk_segments(1000, vec![]);
        assert_eq!(res.segments, vec![DiskSegment::free_space(0, 1000)]);
        assert!(res.anomalies.is_empty());
    }

    #[test]
    fn single_partition_with_trailing_free() {
        let res = compute_disk_segments(1000, vec![part(0, 100, 200)]);
        assert_eq!(
            res.segments,
            vec![
                DiskSegment::free_space(0, 100),
                DiskSegment::partition(0, 100, 200),
                DiskSegment::free_space(300, 700)
            ]
        );
    }

    #[test]
    fn multiple_partitions_with_gaps_and_unsorted_input() {
        let res = compute_disk_segments(1000, vec![part(1, 600, 100), part(0, 100, 200)]);
        assert_eq!(
            res.segments,
            vec![
                DiskSegment::free_space(0, 100),
                DiskSegment::partition(0, 100, 200),
                DiskSegment::free_space(300, 300),
                DiskSegment::partition(1, 600, 100),
                DiskSegment::free_space(700, 300)
            ]
        );
    }

    #[test]
    fn overlapping_partitions_do_not_panic_and_remain_ordered() {
        let res = compute_disk_segments(1000, vec![part(0, 100, 300), part(1, 200, 200)]);
        assert!(
            res.anomalies
                .iter()
                .any(|a| matches!(a, SegmentAnomaly::PartitionOverlapsPrevious { id: 1, .. }))
        );

        // Layout remains ordered and non-overlapping for UI rendering.
        let segments = res.segments;
        for w in segments.windows(2) {
            let a = w[0];
            let b = w[1];
            assert!(a.offset.saturating_add(a.size) <= b.offset);
        }
    }

    #[test]
    fn partition_end_past_disk_is_clamped() {
        let res = compute_disk_segments(1000, vec![part(0, 900, 200)]);
        assert!(
            res.anomalies
                .iter()
                .any(|a| matches!(a, SegmentAnomaly::PartitionEndPastDisk { id: 0, .. }))
        );
        assert_eq!(
            res.segments,
            vec![
                DiskSegment::free_space(0, 900),
                DiskSegment::partition(0, 900, 100)
            ]
        );
    }

    #[test]
    fn extremely_small_partitions_are_preserved_as_non_zero_size() {
        let res = compute_disk_segments(1000, vec![part(0, 10, 1)]);
        assert_eq!(
            res.segments,
            vec![
                DiskSegment::free_space(0, 10),
                DiskSegment::partition(0, 10, 1),
                DiskSegment::free_space(11, 989)
            ]
        );
    }
}
