use std::sync::Arc;
use std::fmt::Debug;

use crate::{employees::{Employee, EmployeeQueryBuilder, EmployeeRelations}, user_preferences::UserPreference};

#[derive(Debug, Clone)]
pub struct User<E = (), P = (), ER = ()> 
where 
    ER: EmployeeRelations + Debug + Clone + 'static,
    ER::Branch:  Debug + Clone,
    ER::User:   Debug + Clone
{
    pub id: i32,
    pub name: &'static str,
    pub email: &'static str,
    pub employee_id: Option<i32>,
    query_results: Option<UserQueryResults<ER>>,
    _relations: std::marker::PhantomData<(E, P)>,
}

impl User {
    pub fn query() -> UserQueryBuilderInitial {
        UserQueryBuilderInitial
    }
}

// Methods available when employee data is loaded
impl<P, ER> User<WithEmployee, P, ER> 
where 
    ER: EmployeeRelations + Debug + Clone + 'static,
    ER::Branch: Debug + Clone,
    ER::User: Debug + Clone
{
    pub fn employee(&self) -> Option<&Employee<ER::Branch, ER::User>> {
        self.query_results
            .as_ref()
            .and_then(|qr| qr.employee.as_ref())
    }
}

// Methods available when preferences are loaded
impl<E, ER> User<E, WithPreferences, ER> 
where 
    ER: EmployeeRelations + Debug + Clone + 'static,
    ER::Branch: Debug + Clone,
    ER::User: Debug + Clone
{
    pub fn preferences(&self) -> Option<&UserPreference> {
        self.query_results
            .as_ref()
            .and_then(|qr| qr.user_preferences.as_ref())
    }
}

#[derive(Debug, Clone)]
struct UserQueryResults<ER = ()> 
where 
    ER: EmployeeRelations + Debug + Clone + 'static
{
    employee: Option<Employee<ER::Branch, ER::User>>,
    user_preferences: Option<UserPreference>,
}

pub struct UserQueryBuilderInitial;

pub trait UserRelations {}
impl UserRelations for () {}
impl UserRelations for WithEmployee {}
impl UserRelations for WithPreferences {}

#[derive(Debug, Clone)]
pub struct WithEmployee;
#[derive(Debug, Clone)]
pub struct WithPreferences;

pub enum UserQueryBuilderCondition {
    ById(i32),
    ByEmployeeId(i32),
}

impl UserQueryBuilderInitial {
    pub fn by_id(&self, id: i32) -> UserQueryBuilder {
        UserQueryBuilder::new(UserQueryBuilderCondition::ById(id))
    }

    pub fn by_employee_id(&self, employee_id: i32) -> UserQueryBuilder {
        UserQueryBuilder::new(UserQueryBuilderCondition::ByEmployeeId(employee_id))
    }
}

pub struct UserQueryBuilder<E = (), P = (), ER = ()> 
where 
    ER: EmployeeRelations + 'static
{
    condition: UserQueryBuilderCondition,
    _employee: std::marker::PhantomData<E>,
    employee_configuration:
        Option<Arc<dyn Fn(EmployeeQueryBuilder) -> EmployeeQueryBuilder<ER::Branch, ER::User> + Send + Sync>>,
    _preferences: std::marker::PhantomData<P>,
}

impl<B, U> UserQueryBuilder<B, U> {
    pub fn new(condition: UserQueryBuilderCondition) -> Self {
        Self {
            condition,
            _employee: std::marker::PhantomData,
            employee_configuration: None,
            _preferences: std::marker::PhantomData,
        }
    }
}

// Split into two impls - one for the builder methods and one for execute
impl<E, P> UserQueryBuilder<E, P>
where
    E: UserRelations,
    P: UserRelations,
{
    pub fn with_employee<F, B, U>(self, configurator: F) -> UserQueryBuilder<WithEmployee, P, (B, U)> 
    where 
        F: Fn(EmployeeQueryBuilder) -> EmployeeQueryBuilder<B, U> + Send + Sync + 'static,
        B: Debug + Clone + Send + Sync + 'static,
        U: Debug + Clone + Send + Sync + 'static,
        (B, U): EmployeeRelations<Branch = B, User = U> + Debug + Clone + 'static
    {
        UserQueryBuilder {
            condition: self.condition,
            _employee: std::marker::PhantomData,
            employee_configuration: Some(Arc::new(configurator)),
            _preferences: std::marker::PhantomData,
        }
    }

    pub fn with_preferences(self) -> UserQueryBuilder<E, WithPreferences> {
        UserQueryBuilder::new(self.condition)
    }
}

// Separate impl for execute that requires the loading traits
impl<E: UserRelations + 'static, P: UserRelations + 'static, ER> UserQueryBuilder<E, P, ER> 
where
    ER: EmployeeRelations + Debug + Clone + 'static,
    ER::Branch: Debug + Clone + Send + Sync + 'static,
    ER::User: Debug + Clone + Send + Sync + 'static,
{
    pub async fn execute(&self) -> Vec<User<E, P, ER>> {
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;

        let mut users = match &self.condition {
            UserQueryBuilderCondition::ById(id) => USER_LIST
                .iter()
                .filter(|e| e.id == *id)
                .map(|u| User {
                    id: u.id,
                    name: u.name,
                    email: u.email,
                    employee_id: u.employee_id,
                    query_results: None,
                    _relations: std::marker::PhantomData,
                })
                .collect::<Vec<_>>(),
            UserQueryBuilderCondition::ByEmployeeId(employee_id) => USER_LIST
                .iter()
                .filter(|e| e.employee_id == Some(*employee_id))
                .map(|u| User {
                    id: u.id,
                    name: u.name,
                    email: u.email,
                    employee_id: u.employee_id,
                    query_results: None,
                    _relations: std::marker::PhantomData,
                })
                .collect::<Vec<_>>(),
        };

        for user in &mut users {
            let mut query_results = UserQueryResults {
                employee: None,
                user_preferences: None,
            };

            // Only load employee data if E is WithEmployee
            if std::any::TypeId::of::<E>() == std::any::TypeId::of::<WithEmployee>() {
                if let Some(employee_id) = user.employee_id {
                    if let Some(ref configurator) = self.employee_configuration {
                        let query = Employee::query().by_id(employee_id);
                        let configured_query = configurator(query);
                        query_results.employee = configured_query.execute_one().await;
                    }
                }
            }

            // Only load preferences if P is WithPreferences
            if std::any::TypeId::of::<P>() == std::any::TypeId::of::<WithPreferences>() {
                query_results.user_preferences = crate::user_preferences::USER_PREFERENCES_LIST
                    .iter()
                    .find(|p| p.user_id == user.id)
                    .cloned();
            }

            user.query_results = Some(query_results);
        }

        users
    }

    pub async fn execute_one(&self) -> Option<User<E, P, ER>> {
        self.execute().await.pop()
    }
}

const USER_LIST: [User; 3] = [
    User {
        id: 1,
        name: "Alice",
        email: "alice@email.com",
        employee_id: Some(1),
        query_results: None,
        _relations: std::marker::PhantomData,
    },
    User {
        id: 2,
        name: "Bob",
        email: "bob@email.com",
        employee_id: Some(2),
        query_results: None,
        _relations: std::marker::PhantomData,
    },
    User {
        id: 3,
        name: "Charlie",
        email: "charlie@email.com",
        employee_id: None,
        query_results: None,
        _relations: std::marker::PhantomData,
    },
];
