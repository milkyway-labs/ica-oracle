use crate::ContractError;
use regex::Regex;
use sha2::{Digest, Sha256};

const CHANNEL_REGEX: &str = r"^channel-\d+$";
const TRANSFER_PORT_ID: &str = "transfer";

/// follows cosmos SDK validation logic where denoms can be 3 - 128 characters long
/// and starts with a letter, followed but either a letter, number, or separator ( ‘/' , ‘:' , ‘.’ , ‘_’ , or '-')
/// reference: https://github.com/cosmos/cosmos-sdk/blob/7728516abfab950dc7a9120caad4870f1f962df5/types/coin.go#L865-L867
pub fn validate_native_denom(denom: &str) -> Result<(), ContractError> {
    if denom.len() < 3 || denom.len() > 128 {
        return Err(ContractError::InvalidDenom {
            reason: "Invalid denom length".to_string(),
        });
    }

    let mut chars = denom.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() {
        return Err(ContractError::InvalidDenom {
            reason: "First character is not ASCII alphabetic".to_string(),
        });
    }

    let set = ['/', ':', '.', '_', '-'];
    for c in chars {
        if !(c.is_ascii_alphanumeric() || set.contains(&c)) {
            return Err(ContractError::InvalidDenom {
                reason: "Not all characters are ASCII alphanumeric or one of:  /  :  .  _  -"
                    .to_string(),
            });
        }
    }

    Ok(())
}

// Validates that the channel ID is of the form `channel-N`
pub fn validate_channel_id(channel_id: &str) -> Result<(), ContractError> {
    let re = Regex::new(CHANNEL_REGEX).unwrap();
    if !re.is_match(channel_id) {
        return Err(ContractError::InvalidChannelID {
            channel_id: channel_id.to_string(),
        });
    }
    Ok(())
}

// Given a base denom and channelID, returns the IBC denom hash
// E.g. base_denom: uosmo, channel_id: channel-0 => ibc/{hash(transfer/channel-0/uosmo)}
// Note: This function only supports ibc denom's that originated on the controller chain
pub fn denom_trace_to_hash(base_denom: &str, channel_id: &str) -> Result<String, ContractError> {
    if base_denom.starts_with("ibc/") {
        return Err(ContractError::InvalidRedemptionRateDenom {
            denom: base_denom.to_string(),
        });
    }

    let denom_trace = format!("{TRANSFER_PORT_ID}/{channel_id}/{base_denom}");

    let mut hasher = Sha256::new();
    hasher.update(denom_trace.as_bytes());
    let result = hasher.finalize();
    let hash = hex::encode(result);

    let ibc_hash = format!("ibc/{}", hash.to_uppercase());
    Ok(ibc_hash)
}

#[cfg(test)]
mod tests {
    use crate::helpers::{denom_trace_to_hash, validate_channel_id, validate_native_denom};
    use crate::ContractError;

    #[test]
    fn length_below_three() {
        let res = validate_native_denom("su");
        assert_eq!(
            res,
            Err(ContractError::InvalidDenom {
                reason: "Invalid denom length".to_string()
            }),
        )
    }

    #[test]
    fn length_above_128() {
        let res =
            validate_native_denom("fadjkvnrufbaalkefoi2934095sfonalf89o234u2sadsafsdbvsdrgweqraefsdgagqawfaf104hqflkqehf98348qfhdsfave3r23152wergfaefegqsacasfasfadvcadfsdsADsfaf324523");
        assert_eq!(
            res,
            Err(ContractError::InvalidDenom {
                reason: "Invalid denom length".to_string()
            }),
        )
    }

    #[test]
    fn first_char_not_alphabetical() {
        let res = validate_native_denom("7asdkjnfe7");
        assert_eq!(
            res,
            Err(ContractError::InvalidDenom {
                reason: "First character is not ASCII alphabetic".to_string()
            }),
        )
    }

    #[test]
    fn invalid_character() {
        let res = validate_native_denom("fakjfh&asd!#");
        assert_eq!(
            res,
            Err(ContractError::InvalidDenom {
                reason: "Not all characters are ASCII alphanumeric or one of:  /  :  .  _  -"
                    .to_string()
            }),
        )
    }

    #[test]
    fn correct_denom() {
        let res = validate_native_denom("umars");
        assert_eq!(res, Ok(()));

        let res = validate_native_denom(
            "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2",
        );
        assert_eq!(res, Ok(()));
    }

    #[test]
    fn test_validate_channel_id() {
        assert_eq!(validate_channel_id("channel-0"), Ok(()));
        assert_eq!(validate_channel_id("channel-100"), Ok(()));
        assert_eq!(validate_channel_id("channel-999"), Ok(()));

        assert_eq!(
            validate_channel_id("channel-"),
            Err(ContractError::InvalidChannelID {
                channel_id: "channel-".to_string()
            })
        );

        assert_eq!(
            validate_channel_id("chan-0"),
            Err(ContractError::InvalidChannelID {
                channel_id: "chan-0".to_string()
            })
        );

        assert_eq!(
            validate_channel_id("Xchannel-0"),
            Err(ContractError::InvalidChannelID {
                channel_id: "Xchannel-0".to_string()
            })
        );

        assert_eq!(
            validate_channel_id("channel-0X"),
            Err(ContractError::InvalidChannelID {
                channel_id: "channel-0X".to_string()
            })
        );
    }

    #[test]
    fn test_denom_trace_to_hash() {
        assert_eq!(
            denom_trace_to_hash("uatom", "channel-0"),
            Ok("ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string()),
        );

        assert_eq!(
            denom_trace_to_hash("uatom", "channel-3"),
            Ok("ibc/A4DB47A9D3CF9A068D454513891B526702455D3EF08FB9EB558C561F9DC2B701".to_string()),
        );

        assert_eq!(
            denom_trace_to_hash("uatom", "channel-10"),
            Ok("ibc/A670D9568B3E399316EEDE40C1181B7AA4BD0695F0B37513CE9B95B977DFC12E".to_string()),
        );

        assert_eq!(
            denom_trace_to_hash("uosmo", "channel-999"),
            Ok("ibc/BBF0BA1A51EA726A21CDC784B4834DCB64407BB6E2BFC8F15DE266DB05F6000D".to_string()),
        );

        assert_eq!(
            denom_trace_to_hash("uusdc", "channel-208"),
            Ok("ibc/D189335C6E4A68B513C10AB227BF1C1D38C746766278BA3EEB4FB14124F1D858".to_string()),
        );

        assert_eq!(
            denom_trace_to_hash("ujuno", "channel-42"),
            Ok("ibc/46B44899322F3CD854D2D46DEEF881958467CDD4B3B10086DA49296BBED94BED".to_string()),
        );

        assert_eq!(
            denom_trace_to_hash("aevmos", "channel-204"),
            Ok("ibc/6AE98883D4D5D5FF9E50D7130F1305DA2FFA0C652D1DD9C123657C6B4EB2DF8A".to_string()),
        );

        assert_eq!(
            denom_trace_to_hash("ustrd", "channel-326"),
            Ok("ibc/A8CA5EE328FA10C9519DF6057DA1F69682D28F7D0F5CCC7ECB72E3DCA2D157A4".to_string()),
        );

        assert_eq!(
            denom_trace_to_hash("stuatom", "channel-326"),
            Ok("ibc/C140AFD542AE77BD7DCC83F13FDD8C5E5BB8C4929785E6EC2F4C636F98F17901".to_string()),
        );

        assert_eq!(
            denom_trace_to_hash("stuosmo", "channel-326"),
            Ok("ibc/D176154B0C63D1F9C6DCFB4F70349EBF2E2B5A87A05902F57A6AE92B863E9AEC".to_string()),
        );

        assert_eq!(
            denom_trace_to_hash("ibc/XXX", "channel-999"),
            Err(ContractError::InvalidRedemptionRateDenom {
                denom: "ibc/XXX".to_string()
            }),
        );
    }
}
