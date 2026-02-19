#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env, Vec, BytesN};

#[test]
fn test_create_project() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PifpProtocol);
    let client = PifpProtocolClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let oracle = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(Address::generate(&env));
    let project_id = BytesN::from_array(&env, &[0u8; 32]);

    let milestones = Vec::from_array(&env, [
        Milestone { id: 1, amount: 500, status: MilestoneStatus::Pending },
        Milestone { id: 2, amount: 500, status: MilestoneStatus::Pending },
    ]);

    let project = client.create_project(&project_id, &creator, &oracle, &token_id, &1000, &milestones);

    assert_eq!(project.goal, 1000);
    assert_eq!(project.creator, creator);
    assert_eq!(project.oracle, oracle);
    assert_eq!(project.milestones.len(), 2);

    let fetched_project = client.get_project(&project_id).unwrap();
    assert_eq!(fetched_project.goal, 1000);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_create_project_goal_mismatch() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PifpProtocol);
    let client = PifpProtocolClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let oracle = Address::generate(&env);
    let token_id = Address::generate(&env);
    let project_id = BytesN::from_array(&env, &[0u8; 32]);

    let milestones = Vec::from_array(&env, [
        Milestone { id: 1, amount: 500, status: MilestoneStatus::Pending },
    ]);

    // Goal is 1000, but milestones sum to 500
    client.create_project(&project_id, &creator, &oracle, &token_id, &1000, &milestones);
}

#[test]
fn test_deposit_and_release() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PifpProtocol);
    let client = PifpProtocolClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let oracle = Address::generate(&env);
    let donator = Address::generate(&env);
    
    // Setup token
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let sac_client = token::StellarAssetClient::new(&env, &token_id);
    let token_client = token::Client::new(&env, &token_id);

    // Mint tokens to donator
    sac_client.mint(&donator, &2000);

    let project_id = BytesN::from_array(&env, &[1u8; 32]);
    let milestones = Vec::from_array(&env, [
        Milestone { id: 1, amount: 400, status: MilestoneStatus::Pending },
        Milestone { id: 2, amount: 600, status: MilestoneStatus::Pending },
    ]);

    client.create_project(&project_id, &creator, &oracle, &token_id, &1000, &milestones);

    // Deposit
    client.deposit(&project_id, &donator, &1000);
    assert_eq!(token_client.balance(&contract_id), 1000);
    assert_eq!(client.get_project(&project_id).unwrap().balance, 1000);

    // Release Milestone 1
    client.release_milestone(&project_id, &1);

    let updated_project = client.get_project(&project_id).unwrap();
    assert_eq!(updated_project.balance, 600);
    assert_eq!(updated_project.milestones.get(0).unwrap().status, MilestoneStatus::Released);
    assert_eq!(token_client.balance(&creator), 400);
    assert_eq!(token_client.balance(&contract_id), 600);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_release_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PifpProtocol);
    let client = PifpProtocolClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let oracle = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(Address::generate(&env));
    
    let project_id = BytesN::from_array(&env, &[2u8; 32]);
    let milestones = Vec::from_array(&env, [
        Milestone { id: 1, amount: 1000, status: MilestoneStatus::Pending },
    ]);

    client.create_project(&project_id, &creator, &oracle, &token_id, &1000, &milestones);

    // No deposit made, try to release
    client.release_milestone(&project_id, &1);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_cannot_release_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PifpProtocol);
    let client = PifpProtocolClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let oracle = Address::generate(&env);
    let donator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let sac_client = token::StellarAssetClient::new(&env, &token_id);

    sac_client.mint(&donator, &1000);

    let project_id = BytesN::from_array(&env, &[3u8; 32]);
    let milestones = Vec::from_array(&env, [
        Milestone { id: 1, amount: 1000, status: MilestoneStatus::Pending },
    ]);

    client.create_project(&project_id, &creator, &oracle, &token_id, &1000, &milestones);
    client.deposit(&project_id, &donator, &1000);

    client.release_milestone(&project_id, &1); // First time
    client.release_milestone(&project_id, &1); // Second time should fail
}
