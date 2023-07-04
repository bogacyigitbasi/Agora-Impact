use concordium_cis2::*;
use concordium_std::*;

use super::{contract_types::*, error::*, state::*, events::*};

pub type TransferParameter = TransferParams<ContractTokenId, ContractTokenAmount>;

/// Execute a list of token transfers, in the order of the list.
///
/// Logs a `Transfer` event and invokes a receive hook function for every
/// transfer in the list.
///
/// It rejects if:
/// - It fails to parse the parameter.
/// - Any of the transfers fail to be executed, which could be if:
///     - The `token_id` does not exist.
///     - The sender is not the owner of the token, or an operator for this
///       specific `token_id` and `from` address.
///     - The token is not owned by the `from`.
/// - Fails to log event.
/// - Any of the receive hook function calls rejects.
#[receive(
    contract = "project_nft",
    name = "transfer",
    parameter = "TransferParameter",
    error = "ContractError",
    enable_logger,
    mutable
)]
pub fn transfer<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>, StateApiType = S>,
    logger: &mut impl HasLogger,
) -> ContractResult<()> {
    // Parse the parameter.
    let TransferParams(transfers): TransferParameter = ctx.parameter_cursor().get()?;
    // Get the sender who invoked this contract function.
    let sender = ctx.sender();

    for Transfer {
        token_id,
        amount,
        from,
        to,
        data: _,
    } in transfers
    {
        let (state, builder) = host.state_and_builder();
        // Authenticate the sender for this transfer
        ensure!(from == sender, ContractError::Unauthorized);
        // Ensure that the amount is 1.
        ensure!(
            amount.cmp(&1.into()).is_eq(),
            ContractError::Custom(CustomContractError::InvalidAmount)
        );
        let to_address = to.address();
        // Update the contract state
        state.transfer(&token_id, &from, &to_address, builder)?;

        // Log transfer event
        logger.log(&ContractEvent::Transfer(TransferEvent {
            token_id,
            amount,
            from,
            to: to_address,
        }))?;

        // Get the token from the state.
        // its safe to unwrap here since we just transferred it.
        let token = state.get_token(&token_id).unwrap();

        let maturity_time_millis: Vec<u8> =
            token.maturity_time.timestamp_millis().to_le_bytes().into();
        // If the receiver is a contract we invoke it.
        if let Receiver::Contract(address, entrypoint_name) = to {
            let parameter = OnReceivingCis2Params {
                token_id,
                amount,
                from,
                // The data field is used to pass the maturity time of the token.
                // Recipient contract can use this to check if the token is mature.
                // Ex. if the token is not mature yet, fractionalizer will not accept it.
                data: AdditionalData::from(maturity_time_millis),
            };
            host.invoke_contract(
                &address,
                &parameter,
                entrypoint_name.as_entrypoint_name(),
                Amount::zero(),
            )?;
        }
    }
    Ok(())
}