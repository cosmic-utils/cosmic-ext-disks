


#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CreatePartitionInfo
{
    pub name: String,
    pub size: u64,
    pub max_size: u64,
    pub offset: u64,
    pub erase: bool,
    pub selected_type: String,
    pub selected_partitition_type: usize,
    pub password_protected: bool,
    pub password: String,
    pub confirmed_password: String,
    pub can_continue: bool,

}
