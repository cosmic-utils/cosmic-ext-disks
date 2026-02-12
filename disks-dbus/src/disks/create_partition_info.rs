#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CreatePartitionInfo {
    pub name: String,
    pub size: u64,
    pub max_size: u64,
    pub offset: u64,
    pub erase: bool,
    pub selected_type: String,
    pub selected_partition_type_index: usize,
    pub password_protected: bool,
    pub password: String,
    pub confirmed_password: String,
    pub can_continue: bool,
    pub filesystem_type: String,
    pub table_type: String,
    pub size_text: String,
    pub size_unit_index: usize,
}
