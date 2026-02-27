use storage_testing::tests;

pub fn has_id(id: &str) -> bool {
    tests::instantiate_tests()
        .iter()
        .any(|entry| entry.id() == id)
}
