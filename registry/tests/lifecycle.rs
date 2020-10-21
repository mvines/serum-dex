use rand::rngs::OsRng;
use serum_common::client::rpc;
use serum_common_tests::Genesis;
use serum_registry::accounts::StakeKind;
use serum_registry_client::*;
use solana_client_gen::prelude::*;
use solana_client_gen::solana_sdk::pubkey::Pubkey;
use solana_client_gen::solana_sdk::signature::{Keypair, Signer};
use spl_token::state::Account as TokenAccount;

// NOTE: Deterministic derived addresses are used as a UX convenience so
//       make sure tests are run against a new instance of the program.

// lifecycle tests all instructions on the program in one go.
// TODO: break this up into multiple tests.
#[test]
fn lifecycle() {
    // First test initiailze.
    let genesis = serum_common_tests::genesis::<Client>();

    let Genesis {
        client,
        srm_mint,
        msrm_mint,
        mint_authority: _,
        god,
        god_msrm: _,
        god_balance_before,
        god_msrm_balance_before: _,
        god_owner,
    } = genesis;

    // Initialize the registrar.
    let withdrawal_timelock = 1234;
    let deactivation_timelock_premium = 1000;
    let reward_activation_threshold = 10_000_000;
    let registrar_authority = Keypair::generate(&mut OsRng);
    let InitializeResponse { registrar, .. } = client
        .initialize(InitializeRequest {
            registrar_authority: registrar_authority.pubkey(),
            withdrawal_timelock,
            deactivation_timelock_premium,
            mint: srm_mint.pubkey(),
            mega_mint: msrm_mint.pubkey(),
            reward_activation_threshold,
        })
        .unwrap();

    // Initialize the lockup program and whitelist registrar.
    {
        let lockup_program_id: Pubkey = std::env::var("TEST_LOCKUP_PROGRAM_ID")
            .unwrap()
            .parse()
            .unwrap();
        // TODO
    }

    // Verify initialization.
    {
        let registrar = client.registrar(&registrar).unwrap();
        assert_eq!(registrar.initialized, true);
        assert_eq!(registrar.authority, registrar_authority.pubkey());
        assert_eq!(registrar.capabilities_fees_bps, [0; 32]);
    }

    // Register capabilities.
    {
        let capability_id = 1;
        let capability_fee_bps = 1234;

        let _ = client
            .register_capability(RegisterCapabilityRequest {
                registrar,
                registrar_authority: &registrar_authority,
                capability_id,
                capability_fee_bps,
            })
            .unwrap();

        let registrar = client.registrar(&registrar).unwrap();
        let mut expected = [0; 32];
        expected[capability_id as usize] = capability_fee_bps;
        assert_eq!(registrar.capabilities_fees_bps, expected);
    }

    // Create entity.
    let node_leader = Keypair::generate(&mut OsRng);
    let node_leader_pubkey = node_leader.pubkey();
    let entity = {
        let capabilities = 1;
        let stake_kind = StakeKind::Delegated;

        let CreateEntityResponse { tx: _, entity } = client
            .create_entity(CreateEntityRequest {
                node_leader: &node_leader,
                capabilities,
                stake_kind,
                registrar,
            })
            .unwrap();
        let entity_acc = client.entity(&entity).unwrap();
        assert_eq!(entity_acc.leader, node_leader_pubkey);
        assert_eq!(entity_acc.initialized, true);
        assert_eq!(entity_acc.balances.amount, 0);
        assert_eq!(entity_acc.balances.mega_amount, 0);
        assert_eq!(entity_acc.capabilities, capabilities);
        assert_eq!(entity_acc.stake_kind, stake_kind);
        entity
    };

    // Update entity.
    {
        let new_capabilities = 1 | 2;
        let new_leader = Pubkey::new_rand();
        let _ = client
            .update_entity(UpdateEntityRequest {
                entity,
                leader: &node_leader,
                new_leader,
                new_capabilities,
            })
            .unwrap();

        let entity_account = client.entity(&entity).unwrap();
        assert_eq!(entity_account.capabilities, new_capabilities);
        assert_eq!(entity_account.leader, new_leader);
    }

    // Join enitty.
    let beneficiary = Keypair::generate(&mut OsRng);
    let member = {
        let JoinEntityResponse { tx: _, member } = client
            .join_entity(JoinEntityRequest {
                entity,
                registrar,
                beneficiary: beneficiary.pubkey(),
                delegate: Pubkey::new_from_array([0; 32]),
                watchtower: Pubkey::new_from_array([0; 32]),
                watchtower_dest: Pubkey::new_from_array([0; 32]),
            })
            .unwrap();

        let member_account = client.member(&member).unwrap();
        assert_eq!(member_account.initialized, true);
        assert_eq!(member_account.entity, entity);
        assert_eq!(member_account.beneficiary, beneficiary.pubkey());
        assert_eq!(
            member_account.books.delegate().owner,
            Pubkey::new_from_array([0; 32])
        );
        assert_eq!(member_account.books.main().balances.amount, 0);
        assert_eq!(member_account.books.main().balances.mega_amount, 0);
        member
    };

    // Stake intent.
    let stake_intent_amount = 33;
    {
        client
            .stake_intent(StakeIntentRequest {
                member,
                beneficiary: &beneficiary,
                entity,
                depositor: god.pubkey(),
                depositor_authority: &god_owner,
                mega: false,
                registrar,
                amount: stake_intent_amount,
            })
            .unwrap();
        let vault = client.stake_intent_vault(&registrar).unwrap();
        assert_eq!(stake_intent_amount, vault.amount);
        let god_acc = rpc::get_token_account::<TokenAccount>(client.rpc(), &god.pubkey()).unwrap();
        assert_eq!(god_acc.amount, god_balance_before - stake_intent_amount);
    }

    // Stake intent withdrawal.
    {
        client
            .stake_intent_withdrawal(StakeIntentWithdrawalRequest {
                member,
                beneficiary: &beneficiary,
                entity,
                depositor: god.pubkey(),
                mega: false,
                registrar,
                amount: stake_intent_amount,
            })
            .unwrap();
        let vault = client.stake_intent_vault(&registrar).unwrap();
        assert_eq!(0, vault.amount);
        let god_acc = rpc::get_token_account::<TokenAccount>(client.rpc(), &god.pubkey()).unwrap();
        assert_eq!(god_acc.amount, god_balance_before);
    }

    // Stake intent from lockup.
    {
        // todo
    }

    // Stake intent withdrawal from delegate.
    {
        // todo
    }

    // Stake transfer.
    {
        // todo
    }

    // Stake.
    {
        // todo
    }

    // Stake withdrawal.
    {
        // todo
    }
}
