#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, token, Address, BytesN,
    Env, Symbol, Vec,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MilestoneStatus {
    Pending = 0,
    Released = 1,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub id: u32,
    pub amount: i128,
    pub status: MilestoneStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Project {
    pub id: BytesN<32>,
    pub creator: Address,
    pub oracle: Address,
    pub token: Address,
    pub goal: i128,
    pub milestones: Vec<Milestone>,
    pub balance: i128,
}

#[contracttype]
pub enum DataKey {
    Project(BytesN<32>),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    ProjectNotFound = 1,
    MilestoneNotFound = 2,
    MilestoneAlreadyReleased = 3,
    InsufficientBalance = 4,
    InvalidMilestones = 5,
    NotAuthorized = 6,
    GoalMismatch = 7,
}

#[contract]
pub struct PifpProtocol;

#[contractimpl]
impl PifpProtocol {
    /// Create a new project with milestones.
    ///
    /// - `creator` must authorize the call.
    /// - `goal` is the target amount to be funded.
    /// - `milestones` defines how funds are released. Sum of milestone amounts must match `goal`.
    /// - `oracle` is the address authorized to release milestones.
    pub fn create_project(
        env: Env,
        id: BytesN<32>,
        creator: Address,
        oracle: Address,
        token: Address,
        goal: i128,
        milestones: Vec<Milestone>,
    ) -> Project {
        creator.require_auth();

        if goal <= 0 {
            panic_with_error!(&env, Error::InvalidMilestones);
        }

        let mut total_milestone_amount = 0;
        for milestone in milestones.iter() {
            total_milestone_amount += milestone.amount;
        }

        if total_milestone_amount != goal {
            panic_with_error!(&env, Error::GoalMismatch);
        }

        let project = Project {
            id: id.clone(),
            creator: creator.clone(),
            oracle,
            token,
            goal,
            milestones,
            balance: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Project(id.clone()), &project);

        // Emit project creation event
        env.events().publish(
            (Symbol::new(&env, "project_created"), id),
            creator,
        );

        project
    }

    /// Deposit funds into a project.
    pub fn deposit(env: Env, project_id: BytesN<32>, donator: Address, amount: i128) {
        donator.require_auth();

        let mut project = Self::get_project(env.clone(), project_id.clone())
            .unwrap_or_else(|| panic_with_error!(&env, Error::ProjectNotFound));

        // Transfer tokens from donator to contract
        let token_client = token::Client::new(&env, &project.token);
        token_client.transfer(&donator, &env.current_contract_address(), &amount);

        project.balance += amount;
        env.storage()
            .persistent()
            .set(&DataKey::Project(project_id.clone()), &project);

        // Emit donation event
        env.events().publish(
            (Symbol::new(&env, "donation_received"), project_id),
            (donator, amount),
        );
    }

    /// Release funds for a specific milestone.
    ///
    /// - `oracle` must authorize the call.
    /// - Milestone must exist and be in `Pending` status.
    /// - Project must have sufficient balance for the milestone amount.
    pub fn release_milestone(env: Env, project_id: BytesN<32>, milestone_id: u32) {
        let mut project = Self::get_project(env.clone(), project_id.clone())
            .unwrap_or_else(|| panic_with_error!(&env, Error::ProjectNotFound));

        project.oracle.require_auth();

        let mut milestone_index = None;
        for i in 0..project.milestones.len() {
            let m = project.milestones.get(i).unwrap();
            if m.id == milestone_id {
                milestone_index = Some(i);
                break;
            }
        }

        let index =
            milestone_index.unwrap_or_else(|| panic_with_error!(&env, Error::MilestoneNotFound));
        let mut milestone = project.milestones.get(index).unwrap();

        if milestone.status == MilestoneStatus::Released {
            panic_with_error!(&env, Error::MilestoneAlreadyReleased);
        }

        if project.balance < milestone.amount {
            panic_with_error!(&env, Error::InsufficientBalance);
        }

        // Update state
        milestone.status = MilestoneStatus::Released;
        project.milestones.set(index, milestone.clone());
        project.balance -= milestone.amount;

        // Release funds to creator
        let token_client = token::Client::new(&env, &project.token);
        token_client.transfer(
            &env.current_contract_address(),
            &project.creator,
            &milestone.amount,
        );

        env.storage()
            .persistent()
            .set(&DataKey::Project(project_id.clone()), &project);

        // Emit milestone release event
        env.events().publish(
            (Symbol::new(&env, "milestone_released"), project_id),
            milestone_id,
        );
    }

    /// Retrieve project details.
    pub fn get_project(env: Env, project_id: BytesN<32>) -> Option<Project> {
        env.storage().persistent().get(&DataKey::Project(project_id))
    }
}
