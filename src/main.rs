use employees::{Employee, WithBranch, WithUser};
use users::User;

mod branches;
mod employees;
mod user_preferences;
mod users;

#[tokio::main]
async fn main() {
    let users_basic = User::query().by_id(1).execute().await;
    process_empty_users(users_basic);

    let user_with_employee_and_preferences = User::query()
        .by_id(1)
        .with_preferences()
        .with_employee(|q| q.with_branch())
        .execute()
        .await;
    process_employee_preference_users(user_with_employee_and_preferences);

    let users_with_with_employee = 
        User::query()
            .by_id(2)
            .with_employee(|q| 
                q.with_user().with_branch()
            )
            .execute()
            .await;
    process_employee_users(users_with_with_employee);

    let users_with_preferences = User::query().by_id(3).with_preferences().execute().await;
    process_preference_users(users_with_preferences);

    let employees_basic = Employee::query().by_id(1).execute().await;
    process_basic_employee(employees_basic);

    let employees_branches = Employee::query().by_id(1).with_branch().execute().await;
    process_branch_employee(employees_branches);

    let employees_with_user_and_branches = Employee::query()
        .by_id(2)
        .with_user()
        .with_branch()
        .execute()
        .await;
    process_user_employee_branch(employees_with_user_and_branches);
}

fn process_empty_users(users: Vec<User>) {
    for user in users {
        println!("{:#?}", user);
    }
}

fn process_employee_users(users: Vec<User<users::WithEmployee, (), (WithBranch, WithUser)>>) {
    for user in users {
        let employee = match user.employee() {
            Some(employee) => employee,
            None => continue,
        };
        println!("Employee {:#?}", employee);
        println!("Employee Branch {:#?}", employee.branch());
        println!("Employee User {:#?}", employee.user());
    }
}

fn process_preference_users(users: Vec<User<(), users::WithPreferences>>) {
    for user in users {
        println!("{:#?}", user.preferences());
        // user.employee()
    }
}

fn process_employee_preference_users(
    users: Vec<User<users::WithEmployee, users::WithPreferences, (WithBranch, ())>>,
) {
    for user in users {
        println!("{:#?}", user.employee());
        println!("{:#?}", user.preferences());
    }
}

fn process_basic_employee(employees: Vec<Employee>) {
    for employee in employees {
        println!("{:#?}", employee);
    }
}

fn process_branch_employee(employees: Vec<Employee<employees::WithBranch>>) {
    for employee in employees {
        println!("{:#?}", employee.branch());
    }
}

fn process_user_employee_branch(
    employees: Vec<Employee<employees::WithBranch, employees::WithUser>>,
) {
    for employee in employees {
        println!("{:#?}", employee.branch());
        println!("{:#?}", employee.user());
    }
}
