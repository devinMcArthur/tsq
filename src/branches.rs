#[derive(Debug, Clone)]
pub struct Branch {
    pub id: i32,
    pub name: &'static str,
}

pub const BRANCH_LIST: [Branch; 2] = [
    Branch { id: 1, name: "HQ" },
    Branch {
        id: 2,
        name: "Branch 2",
    },
];
