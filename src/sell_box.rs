use crate::swap_box::SwapBox;
use ergo_headless_dapp_framework::{encoding, *};
/*
==================================================
SellBox ErgoScript
==================================================
    * @notice This is a script for a box that holds an arbitrary token to
    *         be traded for ERG.
    * @param orderOwner Address which owns the SellBox order {Base58 String}
    * @param orderAmount Amount of ERG to be payed to orderOwner {Long}


    {
        val defined = {
            SELF.R4[Coll[Byte]].isDefined &
            SELF.R5[Long].isDefined
        }

        val orderOwner = SELF.R4[Coll[Byte]].get
        val orderAmount = SELF.R5[Long].get


        def correctPayout = {(outBox: Box) =>
            payoutBox.propbytes == SigmaProp(orderOwner) &&
            payoutBox.value >= orderAmount               &&
            outBox.R4[Coll[Byte]].isDefined              &&
            payoutBox.R4[Coll[Byte]].get == SELF.id
        }

        SigmaProp(
            correctPayoutAddress &&
            correctPayoutAmount &&
            correctInput )
    }

==================================================
==================================================
*/

#[derive(Debug, Clone, WrapBox, SpecBox)]
pub struct SellBox {
    ergo_box: ErgoBox,
}

impl SellBox {
    pub const CONTRACT_ADDRESS: &'static str = "PLACEHOLDER";
}

impl SpecifiedBox for SellBox {
    fn box_spec() -> BoxSpec {
        let order_owner = RegisterSpec::new(Some(SType::SColl(Box::new(SType::SByte))), None);
        let order_amount = RegisterSpec::new(Some(SType::SLong), None);

        BoxSpec::new(
            Some(Self::CONTRACT_ADDRESS.to_string()),
            Some(1..u64::MAX),
            vec![order_owner, order_amount],
            vec![],
        )
    }
}

pub struct SellProtocol {}

impl SellProtocol {
    /// Creates an instance of a SellBox
    pub fn action_create_sell_box(
        order_owner: String,
        order_amount: u64,
        ergs_box_to_sell: ErgsBox,
        ergs_box_for_fees: ErgsBox,
        current_height: u64,
        tx_fee: u64,
    ) -> UnsignedTransaction {
        let tx_inputs = vec![
            ergs_box_to_sell.as_unsigned_input(),
            ergs_box_for_fees.as_unsigned_input(),
        ]
        .try_into()
        .unwrap();

        let _order_owner = encoding::serialize_string(&order_owner);
        let _order_amount = Constant::from(order_amount as i64);

        let sell_box_candidate = create_candidate(
            ergs_box_to_sell.nano_ergs(),
            &SellBox::CONTRACT_ADDRESS.to_string(),
            &vec![],
            &vec![_order_owner, _order_amount],
            current_height,
        )
        .unwrap();

        let tx_fee_candidate = TxFeeBox::output_candidate(tx_fee, current_height).unwrap();

        let output_candidates = vec![sell_box_candidate, tx_fee_candidate]
            .try_into()
            .unwrap();

        UnsignedTransaction::new(tx_inputs, None, output_candidates).unwrap()
    }

    /// Refunds the value in a SellBox to it's order_owner
    pub fn action_refund_sell_box(
        sell_box: SellBox,
        tx_fee: u64,
        current_height: u64,
    ) -> UnsignedTransaction {
        let tx_inputs = vec![sell_box.as_unsigned_input()].try_into().unwrap();

        let order_owner = encoding::unwrap_string(&sell_box.registers()[0]).unwrap();

        let value_after_fees = sell_box.nano_ergs() - tx_fee;

        let refund_candidate = create_candidate(
            value_after_fees,
            &order_owner,
            &vec![],
            &vec![],
            current_height,
        )
        .unwrap();

        let tx_fee_candidate = TxFeeBox::output_candidate(tx_fee, current_height).unwrap();

        let output_candidates = vec![refund_candidate, tx_fee_candidate].try_into().unwrap();

        UnsignedTransaction::new(tx_inputs, None, output_candidates).unwrap()
    }

    pub fn action_execute_sell_box(
        sell_box: SellBox,
        ergs_box_to_fufill_order: ErgsBox,
        tx_fee: u64,
        current_height: u64,
    ) -> UnsignedTransaction {
        let tx_inputs = vec![
            sell_box.as_unsigned_input(),
            ergs_box_to_fufill_order.as_unsigned_input(),
        ]
        .try_into()
        .unwrap();

        let order_owner = encoding::unwrap_string(&sell_box.registers()[0]).unwrap();

        let sell_box_id = encoding::serialize_string(&sell_box.box_id());
        let value_after_fees = sell_box.nano_ergs() - tx_fee;

        let sell_box_fufilling_candidate = create_candidate(
            ergs_box_to_fufill_order.nano_ergs(),
            &order_owner,
            &vec![],
            &vec![sell_box_id],
            current_height,
        )
        .unwrap();

        let fufiller_address = ergs_box_to_fufill_order.p2s_address();

        let fufiller_reward_candidate = create_candidate(
            value_after_fees,
            &fufiller_address,
            &sell_box.tokens(),
            &vec![],
            current_height,
        )
        .unwrap();

        let tx_fee_candidate = TxFeeBox::output_candidate(tx_fee, current_height).unwrap();

        let output_candidates = vec![
            sell_box_fufilling_candidate,
            fufiller_reward_candidate,
            tx_fee_candidate,
        ]
        .try_into()
        .unwrap();

        UnsignedTransaction::new(tx_inputs, None, output_candidates).unwrap()
    }

    pub fn get_sell_box_match_spec(sell_box: SellBox) -> BoxSpec {
        let order_amount = encoding::unwrap_long(&sell_box.registers()[1]).unwrap() as u64;

        let own_token = sell_box.tokens()[0].clone();
        let own_token_id = encoding::serialize_string(
            &String::from_utf8(own_token.token_id.as_ref().iter().copied().collect()).unwrap(),
        );

        BoxSpec::new(
            Some(SwapBox::CONTRACT_ADDRESS.to_string()),
            Some(order_amount..u64::MAX),
            vec![
                RegisterSpec::new(Some(SType::SColl(Box::new(SType::SByte))), None),
                RegisterSpec::new(
                    Some(SType::SColl(Box::new(SType::SByte))),
                    Some(own_token_id),
                ),
                RegisterSpec::new(Some(SType::SLong), None),
            ],
            vec![],
        )
    }
}
