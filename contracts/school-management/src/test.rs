#![cfg(test)]

use soroban_sdk::{testutils::Address as _, token, Address, Env, String};

use crate::{
    school_management::{SchoolManagement, SchoolManagementClient},
    storage::Class,
};

fn create_token_contract<'a>(
    env: &Env,
    admin: Address,
) -> (Address, token::StellarAssetClient<'a>) {
    let contract_id = env.register_stellar_asset_contract_v2(admin.clone());
    (
        contract_id.address(),
        token::StellarAssetClient::new(env, &contract_id.address()),
    )
}

struct SetUpResult<'a> {
    env: Env,
    client: SchoolManagementClient<'a>,
    admin: Address,
    student_wallet: Address,
    usdc_asset: Address,
    token_client: token::StellarAssetClient<'a>,
}

fn setup<'a>() -> SetUpResult<'a> {
    let env = Env::default();

    env.mock_all_auths();

    let admin = Address::generate(&env);

    let (usdc_asset, token_client) = create_token_contract(&env, admin.clone());

    let contract_id = env.register(SchoolManagement, (&admin, &usdc_asset));

    let client = SchoolManagementClient::new(&env, &contract_id);

    let student_wallet = Address::generate(&env);

    SetUpResult {
        env,
        client,
        admin,
        student_wallet,
        usdc_asset,
        token_client,
    }
}

// ─── Existing tests ───────────────────────────────────────────────────────────

#[test]
fn test_register_student() {
    let setup_result = setup();

    let name = String::from_str(&setup_result.env, "Sib");

    let class_name = Class::College;

    let registration_result =
        setup_result
            .client
            .register_student(&setup_result.student_wallet, &name, &class_name);

    assert_eq!(registration_result, 1);
}

#[test]
fn test_get_student() {
    let setup_result = setup();

    let name = String::from_str(&setup_result.env, "Sib");

    let class_name = Class::College;

    setup_result
        .client
        .register_student(&setup_result.student_wallet, &name, &class_name);

    let student_id = 1;

    let result = setup_result.client.get_student(&student_id);

    assert_eq!(result.student_id, 1);
    assert_eq!(result.name, name);
}

#[test]
fn test_make_payment() {
    let setup_result = setup();

    let name = String::from_str(&setup_result.env, "Sib");

    let class_name = Class::College;

    setup_result
        .client
        .register_student(&setup_result.student_wallet, &name, &class_name);

    let student_id = 1;

    let amount = 1_000_000i128;

    setup_result
        .token_client
        .mint(&setup_result.student_wallet, &amount);

    let result = setup_result.client.try_make_payment(&student_id, &amount);

    assert!(result.is_ok());

    let student = setup_result.client.get_student(&student_id);

    assert_eq!(student.total_paid, amount);
}

// ─── Update student class ─────────────────────────────────────────────────────

#[test]
fn test_update_student_class() {
    let s = setup();

    let name = String::from_str(&s.env, "Alice");
    s.client.register_student(&s.student_wallet, &name, &Class::Grade);

    let student_id = 1u64;

    // Verify initial class
    let before = s.client.get_student(&student_id);
    assert!(matches!(before.class_name, Class::Grade));

    // Admin promotes the student to HighSchool
    let result = s.client.try_update_student_class(&student_id, &Class::HighSchool);
    assert!(result.is_ok());

    let after = s.client.get_student(&student_id);
    assert!(matches!(after.class_name, Class::HighSchool));
}

#[test]
fn test_update_student_class_to_college() {
    let s = setup();

    let name = String::from_str(&s.env, "Bob");
    s.client.register_student(&s.student_wallet, &name, &Class::HighSchool);

    let student_id = 1u64;

    s.client.update_student_class(&student_id, &Class::College);

    let student = s.client.get_student(&student_id);
    assert!(matches!(student.class_name, Class::College));
}

#[test]
fn test_update_class_fails_for_removed_student() {
    let s = setup();

    let name = String::from_str(&s.env, "Charlie");
    s.client.register_student(&s.student_wallet, &name, &Class::Grade);

    let student_id = 1u64;

    // Remove the student first
    s.client.remove_student(&student_id);

    // Updating class on removed student should fail
    let result = s.client.try_update_student_class(&student_id, &Class::College);
    assert!(result.is_err());
}

// ─── Get payment history ──────────────────────────────────────────────────────

#[test]
fn test_get_payment_history_empty_on_registration() {
    let s = setup();

    let name = String::from_str(&s.env, "Alice");
    s.client.register_student(&s.student_wallet, &name, &Class::College);

    let history = s.client.get_payment_history(&1u64);
    assert_eq!(history.len(), 0);
}

#[test]
fn test_get_payment_history_records_payments() {
    let s = setup();

    let name = String::from_str(&s.env, "Alice");
    s.client.register_student(&s.student_wallet, &name, &Class::College);

    let student_id = 1u64;
    let amount = 500_000i128;

    s.token_client.mint(&s.student_wallet, &(amount * 3));

    s.client.make_payment(&student_id, &amount);
    s.client.make_payment(&student_id, &amount);
    s.client.make_payment(&student_id, &amount);

    let history = s.client.get_payment_history(&student_id);

    assert_eq!(history.len(), 3);
    assert_eq!(history.get(0).unwrap().amount, amount);
    assert_eq!(history.get(1).unwrap().amount, amount);
    assert_eq!(history.get(2).unwrap().amount, amount);
}

#[test]
fn test_payment_history_total_matches_student_total_paid() {
    let s = setup();

    let name = String::from_str(&s.env, "Alice");
    s.client.register_student(&s.student_wallet, &name, &Class::College);

    let student_id = 1u64;

    s.token_client.mint(&s.student_wallet, &2_000_000i128);

    s.client.make_payment(&student_id, &800_000i128);
    s.client.make_payment(&student_id, &1_200_000i128);

    let history = s.client.get_payment_history(&student_id);
    let total_from_history: i128 = history.iter().map(|p| p.amount).sum();

    let student = s.client.get_student(&student_id);

    assert_eq!(total_from_history, student.total_paid);
}

// ─── Remove student ───────────────────────────────────────────────────────────

#[test]
fn test_remove_student_marks_as_not_registered() {
    let s = setup();

    let name = String::from_str(&s.env, "Alice");
    s.client.register_student(&s.student_wallet, &name, &Class::College);

    let student_id = 1u64;

    let result = s.client.try_remove_student(&student_id);
    assert!(result.is_ok());

    let student = s.client.get_student(&student_id);
    assert!(!student.is_registered);
}

#[test]
fn test_remove_student_preserves_payment_history() {
    let s = setup();

    let name = String::from_str(&s.env, "Alice");
    s.client.register_student(&s.student_wallet, &name, &Class::College);

    let student_id = 1u64;
    let amount = 1_000_000i128;

    s.token_client.mint(&s.student_wallet, &amount);
    s.client.make_payment(&student_id, &amount);

    s.client.remove_student(&student_id);

    // Payment history must still be accessible after removal
    let history = s.client.get_payment_history(&student_id);
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().amount, amount);
}

#[test]
fn test_remove_student_twice_fails() {
    let s = setup();

    let name = String::from_str(&s.env, "Alice");
    s.client.register_student(&s.student_wallet, &name, &Class::College);

    let student_id = 1u64;

    s.client.remove_student(&student_id);

    // Second removal should return an error
    let result = s.client.try_remove_student(&student_id);
    assert!(result.is_err());
}
