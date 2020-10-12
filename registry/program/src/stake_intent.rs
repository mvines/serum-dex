use serum_common::pack::Pack;
use serum_registry::access_control;
use serum_registry::accounts::{Entity, Member, Registrar};
use serum_registry::error::{RegistryError, RegistryErrorCode};
use solana_sdk::account_info::{next_account_info, AccountInfo};
use solana_sdk::info;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::sysvar::clock::Clock;

pub fn handler<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    amount: u64,
    is_mega: bool,
    is_delegate: bool,
) -> Result<(), RegistryError> {
    info!("handler: stake");

    let acc_infos = &mut accounts.iter();

    // Lockup whitelist relay interface.

    let depositor_tok_acc_info = next_account_info(acc_infos)?;
    let vault_acc_info = next_account_info(acc_infos)?;
    // Owner or delegate.
    let depositor_tok_authority_acc_info = next_account_info(acc_infos)?;
    let token_program_acc_info = next_account_info(acc_infos)?;

    // Program specfic.

    let member_acc_info = next_account_info(acc_infos)?;
    let member_authority_acc_info = next_account_info(acc_infos)?;
    let entity_acc_info = next_account_info(acc_infos)?;
    let registrar_acc_info = next_account_info(acc_infos)?;
    let clock_acc_info = next_account_info(acc_infos)?;

    access_control(AccessControlRequest {
        depositor_tok_authority_acc_info,
        depositor_tok_acc_info,
        member_acc_info,
        member_authority_acc_info,
        entity_acc_info,
        vault_acc_info,
        token_program_acc_info,
        is_delegate,
        is_mega,
        program_id,
        registrar_acc_info,
    })?;

    Entity::unpack_mut(
        &mut entity_acc_info.try_borrow_mut_data()?,
        &mut |entity: &mut Entity| {
            Member::unpack_mut(
                &mut member_acc_info.try_borrow_mut_data()?,
                &mut |member: &mut Member| {
                    let clock = access_control::clock(clock_acc_info)?;
                    let registrar = Registrar::unpack(&registrar_acc_info.try_borrow_data()?)?;
                    state_transition(StateTransitionRequest {
                        entity,
                        member,
                        amount,
                        registrar,
                        clock,
                        vault_acc_info,
                        depositor_tok_authority_acc_info,
                        depositor_tok_acc_info,
                        member_acc_info,
                        member_authority_acc_info,
                        entity_acc_info,
                        token_program_acc_info,
                        is_delegate,
                        is_mega,
                    })
                    .map_err(Into::into)
                },
            )
        },
    )?;

    Ok(())
}

fn access_control(req: AccessControlRequest) -> Result<(), RegistryError> {
    info!("access-control: stake");

    let AccessControlRequest {
        depositor_tok_authority_acc_info,
        depositor_tok_acc_info,
        member_acc_info,
        member_authority_acc_info,
        entity_acc_info,
        vault_acc_info,
        token_program_acc_info,
        registrar_acc_info,
        program_id,
        is_delegate,
        is_mega,
    } = req;

    // Beneficiary (or delegate) authorization.
    if !depositor_tok_authority_acc_info.is_signer {
        return Err(RegistryErrorCode::Unauthorized)?;
    }

    // Account validation.
    let registrar = access_control::registrar(registrar_acc_info, program_id)?;
    let _ = access_control::entity(entity_acc_info, registrar_acc_info, program_id)?;
    let member = access_control::member(
        member_acc_info,
        entity_acc_info,
        member_authority_acc_info,
        is_delegate,
        program_id,
    )?;
    let _ = access_control::vault(vault_acc_info, &registrar, is_mega)?;

    // StakeIntent specific: None.

    info!("access-control: success");

    Ok(())
}

fn state_transition(req: StateTransitionRequest) -> Result<(), RegistryError> {
    info!("state-transition: stake");

    let StateTransitionRequest {
        entity,
        member,
        amount,
        registrar,
        clock,
        depositor_tok_authority_acc_info,
        depositor_tok_acc_info,
        vault_acc_info,
        member_acc_info,
        member_authority_acc_info,
        entity_acc_info,
        token_program_acc_info,
        is_delegate,
        is_mega,
    } = req;

    // Transfer funds into the stake intent vault.
    {
        info!("invoking token transfer");
        let withdraw_instruction = spl_token::instruction::transfer(
            &spl_token::ID,
            depositor_tok_acc_info.key,
            vault_acc_info.key,
            depositor_tok_authority_acc_info.key,
            &[],
            amount,
        )?;
        solana_sdk::program::invoke_signed(
            &withdraw_instruction,
            &[
                depositor_tok_acc_info.clone(),
                vault_acc_info.clone(),
                depositor_tok_authority_acc_info.clone(),
                token_program_acc_info.clone(),
            ],
            &[],
        )?;
    }

    member.add_stake_intent(amount, is_mega, is_delegate);
    entity.add_stake_intent(amount, is_mega, &registrar, &clock);

    info!("state-transition: success");

    Ok(())
}

struct AccessControlRequest<'a> {
    registrar_acc_info: &'a AccountInfo<'a>,
    program_id: &'a Pubkey,
    depositor_tok_authority_acc_info: &'a AccountInfo<'a>,
    depositor_tok_acc_info: &'a AccountInfo<'a>,
    member_acc_info: &'a AccountInfo<'a>,
    member_authority_acc_info: &'a AccountInfo<'a>,
    entity_acc_info: &'a AccountInfo<'a>,
    token_program_acc_info: &'a AccountInfo<'a>,
    vault_acc_info: &'a AccountInfo<'a>,
    is_delegate: bool,
    is_mega: bool,
}

struct StateTransitionRequest<'a, 'b> {
    entity: &'b mut Entity,
    member: &'b mut Member,
    is_mega: bool,
    is_delegate: bool,
    registrar: Registrar,
    clock: Clock,
    amount: u64,
    vault_acc_info: &'a AccountInfo<'a>,
    depositor_tok_authority_acc_info: &'a AccountInfo<'a>,
    depositor_tok_acc_info: &'a AccountInfo<'a>,
    member_acc_info: &'a AccountInfo<'a>,
    member_authority_acc_info: &'a AccountInfo<'a>,
    entity_acc_info: &'a AccountInfo<'a>,
    token_program_acc_info: &'a AccountInfo<'a>,
}
