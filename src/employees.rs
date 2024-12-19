use crate::{branches::Branch, users::User};

#[derive(Debug, Clone)]
pub struct Employee<B = (), U = ()> {
    pub id: i32,
    pub title: &'static str,
    pub branch_id: i32,
    query_results: Option<EmployeeQueryResults>,
    _relations: std::marker::PhantomData<(B, U)>,
}

impl Employee {
    pub fn query() -> EmployeeQueryBuilderInitial {
        EmployeeQueryBuilderInitial
    }
}

#[derive(Debug, Clone)]
pub struct EmployeeQueryResults {
    branch: Option<Branch>,
    user: Option<Box<User>>,
}

pub trait EmployeeRelations {
    type Branch;
    type User;
}
impl EmployeeRelations for () {
    type Branch = ();
    type User = ();
}
impl EmployeeRelations for WithBranch {
    type Branch = WithBranch;
    type User = ();
}
impl EmployeeRelations for WithUser {
    type Branch = ();
    type User = WithUser;
}
impl<B, U> EmployeeRelations for (B, U) {
    type Branch = B;
    type User = U;
}

#[derive(Debug, Clone)]
pub struct WithBranch;
#[derive(Debug, Clone)]
pub struct WithUser;

impl<U> Employee<WithBranch, U> {
    pub fn branch(&self) -> Option<&Branch> {
        self.query_results
            .as_ref()
            .and_then(|qr| qr.branch.as_ref())
    }
}

impl<B> Employee<B, WithUser> {
    pub fn user(&self) -> Option<&Box<User>> {
        self.query_results
            .as_ref()
            .and_then(|qr| qr.user.as_ref())
    }
}


pub struct EmployeeQueryBuilderInitial;

#[derive(Clone)]
pub enum EmployeeQueryBuilderCondition {
    ById(i32),
}

impl EmployeeQueryBuilderInitial {
    pub fn by_id(&self, id: i32) -> EmployeeQueryBuilder {
        EmployeeQueryBuilder::new(EmployeeQueryBuilderCondition::ById(id))
    }
}

#[derive(Clone)]
pub struct EmployeeQueryBuilder<B = (), U = ()> {
    condition: EmployeeQueryBuilderCondition,
    _relations: std::marker::PhantomData<(B, U)>,
}

impl EmployeeQueryBuilder<(), ()> {
    pub fn new(condition: EmployeeQueryBuilderCondition) -> Self {
        Self {
            condition,
            _relations: std::marker::PhantomData,
        }
    }
}

impl<B, U> EmployeeQueryBuilder<B, U> {
    pub fn with_branch(self) -> EmployeeQueryBuilder<WithBranch, U> {
        EmployeeQueryBuilder {
            condition: self.condition,
            _relations: std::marker::PhantomData,
        }
    }

    pub fn with_user(self) -> EmployeeQueryBuilder<B, WithUser> {
        EmployeeQueryBuilder {
            condition: self.condition,
            _relations: std::marker::PhantomData,
        }
    }
}

impl<B: 'static, U: 'static> EmployeeQueryBuilder<B, U> {
    pub async fn execute(&self) -> Vec<Employee<B, U>> {
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;

        let mut employees = match &self.condition {
            EmployeeQueryBuilderCondition::ById(id) => EMPLOYEE_LIST
                .iter()
                .filter(|e| e.id == *id)
                .map(|u| Employee {
                    id: u.id,
                    title: u.title,
                    branch_id: u.branch_id,
                    query_results: None,
                    _relations: std::marker::PhantomData,
                })
                .collect::<Vec<_>>(),
        };

        for employee in &mut employees {
            let mut query_results = EmployeeQueryResults { branch: None, user: None };

            if std::any::TypeId::of::<B>() == std::any::TypeId::of::<WithBranch>() {
                query_results.branch = crate::branches::BRANCH_LIST
                    .iter()
                    .find(|e| e.id == employee.branch_id)
                    .cloned();
            }

            if std::any::TypeId::of::<U>() == std::any::TypeId::of::<WithUser>() {
                query_results.user = 
                    User::query()
                        .by_employee_id(employee.id)
                        .execute_one()
                        .await
                        .map(Box::new);
            }

            employee.query_results = Some(query_results);
        }

        employees
    }

    pub async fn execute_one(&self) -> Option<Employee<B, U>> {
        Box::pin(async move {
            let mut results = self.execute().await;
            if results.len() > 0 {
                Some(results.remove(0))
            } else {
                None
            }
        }).await
    }
}

pub const EMPLOYEE_LIST: [Employee; 3] = [
    Employee {
        id: 1,
        title: "CEO",
        branch_id: 1,
        query_results: None,
        _relations: std::marker::PhantomData,
    },
    Employee {
        id: 2,
        title: "CTO",
        branch_id: 1,
        query_results: None,
        _relations: std::marker::PhantomData,
    },
    Employee {
        id: 3,
        title: "CFO",
        branch_id: 2,
        query_results: None,
        _relations: std::marker::PhantomData,
    },
];
