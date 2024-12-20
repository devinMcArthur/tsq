use std::sync::Arc;
use std::fmt::Debug;

use crate::{employees::{Employee, EmployeeQueryBuilder, EmployeeRelations}, user_preferences::UserPreference};

#[derive(Debug, Clone)]
pub struct User<E = (), P = ()> 
where 
    E: UserRelations,
    P: UserRelations,
{
    pub id: i32,
    pub name: &'static str,
    pub email: &'static str,
    pub employee_id: Option<i32>,
    query_results: Option<UserQueryResults<E::Employee>>,
    _relations: std::marker::PhantomData<(E, P)>,
}

impl User {
    pub fn query() -> UserQueryBuilderInitial {
        UserQueryBuilderInitial
    }
}

// Methods available when employee data is loaded
impl<P, ER> User<WithEmployee<ER>, P> 
where 
    P: UserRelations,
    ER: EmployeeRelations + Debug + Clone + 'static,
{
    pub fn employee(&self) -> Option<&Employee<ER::Branch, ER::User>> {
        self.query_results
            .as_ref()
            .and_then(|qr| qr.employee.as_ref())
    }
}

// Methods available when preferences are loaded
impl<E> User<E, WithPreferences> 
where 
    E: UserRelations,
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

pub trait UserRelations {
    type Employee: EmployeeRelations + Debug + Clone + 'static;
    type Preferences;
}
impl UserRelations for () {
    type Employee = ();
    type Preferences = ();
}
impl<ER: EmployeeRelations> UserRelations for WithEmployee<ER> 
where 
    ER: EmployeeRelations + Debug + Clone + 'static
{
    type Employee = ER;
    type Preferences = ();
}
impl UserRelations for WithPreferences {
    type Employee = ();
    type Preferences = WithPreferences;
}

#[derive(Debug, Clone)]
pub struct WithEmployee<ER = ()>(std::marker::PhantomData<ER>);
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

pub struct UserQueryBuilder<E = (), P = ()> 
where 
    E: UserRelations,
    P: UserRelations,
{
    condition: UserQueryBuilderCondition,
    _employee: std::marker::PhantomData<E>,
    employee_configuration:
        Option<Arc<dyn Fn(EmployeeQueryBuilder) -> EmployeeQueryBuilder<<E::Employee as EmployeeRelations>::Branch, <E::Employee as EmployeeRelations>::User> + Send + Sync>>,
    _preferences: std::marker::PhantomData<P>,
}

// Split into two impls - one for the builder methods and one for execute
impl<E, P> UserQueryBuilder<E, P>
where
    E: UserRelations,
    P: UserRelations,
{
    pub fn new(condition: UserQueryBuilderCondition) -> Self {
        Self {
            condition,
            _employee: std::marker::PhantomData,
            employee_configuration: None,
            _preferences: std::marker::PhantomData,
        }
    }

    pub fn with_employee<F, ER>(self, configurator: F) -> UserQueryBuilder<WithEmployee<ER>, P> 
    where 
        F: Fn(EmployeeQueryBuilder) -> EmployeeQueryBuilder<ER::Branch, ER::User> + Send + Sync + 'static,
        ER: EmployeeRelations + Debug + Clone + 'static
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
impl<E: UserRelations + 'static, P: UserRelations + 'static> UserQueryBuilder<E, P> 
{
    pub async fn execute(&self) -> Vec<User<E, P>> {
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

            if let Some(employee_id) = user.employee_id {
                if let Some(ref configurator) = self.employee_configuration {
                    let query = Employee::query().by_id(employee_id);
                    let configured_query = configurator(query);
                    query_results.employee = configured_query.execute_one().await;
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

    pub async fn execute_one(&self) -> Option<User<E, P>> {
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
