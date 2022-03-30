use ergo_headless_dapp_framework::{encoding, *};

/*
======================================================================
SWAPBOX ERGOSCRIPT
======================================================================
    * @notice This box holds a token to be swapped for an arbitrary token that isn't Erg or can be refunded
    *         to orderOwner.
    * @param orderOwner Address which owns the SwapBox order {Base58 String}.
    * @param orderTokenId TokenId for token to be payed to orderOwner{Base16 String}.
    * @param orderAmount Amount of orderTokenId to be payed to orderOwner {Long}.

    {
       \// Checks to make sure necessary data is in registers
        val defined = {
            SELF.R4[Coll[Byte]].isDefined          &&
            SELF.R5[(Coll[Byte], Long)].isDefined &&
            SELF.R6[Long].isDefined
        }

        val orderOwner = SELF.R4[Coll[Byte]].get
        val orderTokenId = SELF.R5[Coll[Byte]].get
        val ordeerAmount = SELF.R6[Long].get

        \// Checks to  see that an output box satisfies swap conditions
        def correctPayout = {(outBox: Box) =>
            outBox.tokens(0)._1 == orderToken._1 &&
            outBox.tokens(0)._2 == orderToken._2 &&
            outBox.R4[Coll[Byte]].isDefined      &&
            outBox.R4[Coll[Byte]].get == SELF.id
        }

        sigmaProp((OUTPUTS.exists correctPayout || orderOwner) && defined)
    }
======================================================================
======================================================================
*/

/// BoxSpec for SwapBox
#[derive(Debug, Clone, WrapBox, SpecBox)]
pub struct SwapBox {
    ergo_box: ErgoBox,
}

impl SwapBox {
    pub const CONTRACT_ADDRESS: &'static str = "THIS IS RUBBISH";
}

impl SpecifiedBox for SwapBox {
    fn box_spec() -> BoxSpec {
        let owner_address = RegisterSpec::new(Some(SType::SColl(Box::new(SType::SByte))), None);
        let order_token_id = RegisterSpec::new(Some(SType::SColl(Box::new(SType::SByte))), None);
        let order_token_amount = RegisterSpec::new(Some(SType::SLong), None);

        BoxSpec::new(
            Some(Self::CONTRACT_ADDRESS.to_string()),
            None,
            vec![owner_address, order_token_id, order_token_amount],
            vec![],
        )
    }
}

pub struct SwapProtocol {}

impl SwapProtocol {
    /// Action to create SwapBox instance.
    pub fn action_create_swap_box(
        _order_owner: String,
        _order_token_id: String,
        _order_amount: u64,
        ergs_box_for_swap: ErgsBox,
        current_height: u64,
        transaction_fee: u64,
    ) -> UnsignedTransaction {
        let tx_inputs = vec![ergs_box_for_swap.as_unsigned_input()]
            .try_into()
            .unwrap();

        let value_after_fees = ergs_box_for_swap.nano_ergs() - transaction_fee;

        let order_owner = encoding::serialize_string(&_order_owner);
        let order_token_id = encoding::serialize_string(&_order_token_id);
        let order_amount = Constant::from(_order_amount as i64);

        let swap_box_candidate = create_candidate(
            value_after_fees,
            &SwapBox::CONTRACT_ADDRESS.to_string(),
            &ergs_box_for_swap.tokens(),
            &vec![order_owner, order_token_id, order_amount],
            current_height,
        )
        .unwrap();

        let transaction_fee_candidate =
            TxFeeBox::output_candidate(transaction_fee, current_height).unwrap();

        let output_candidates = vec![swap_box_candidate, transaction_fee_candidate]
            .try_into()
            .unwrap();

        UnsignedTransaction::new(tx_inputs, None, output_candidates).unwrap()
    }

    /// Action to reclaim SwapBox instance by orderOwner :(
    pub fn action_reclaim_swap(
        order_owner: String,
        swap_box_to_reclaim: SwapBox,
        current_height: u64,
        transaction_fee: u64,
    ) -> UnsignedTransaction {
        let tx_inputs = vec![swap_box_to_reclaim.as_unsigned_input()]
            .try_into()
            .unwrap();

        let refund_value_after_fees = swap_box_to_reclaim.nano_ergs() - transaction_fee;

        let refund_candidate = create_candidate(
            refund_value_after_fees,
            &order_owner,
            &swap_box_to_reclaim.tokens(),
            &vec![],
            current_height,
        )
        .unwrap();

        let transaction_fee_candidate =
            TxFeeBox::output_candidate(transaction_fee, current_height).unwrap();

        let output_candidates = vec![refund_candidate, transaction_fee_candidate]
            .try_into()
            .unwrap();

        UnsignedTransaction::new(tx_inputs, None, output_candidates).unwrap()
    }

    /// Takes two boxes that can fufill each other and execute swap :)
    /*
     * @param swap_box SwapBox to be fufilled.
     * @param ergs_box_to_fufill ErgsBox to be used in tx to fufill swap_box order.
     * @param fufiller_address Address to be payed reward from swap_box order.
     */
    pub fn action_execute_swap(
        swap_box_1: SwapBox,
        swap_box_2: SwapBox,
        current_height: u64,
        transaction_fee: u64,
    ) -> UnsignedTransaction {
        let tx_inputs = vec![
            swap_box_1.as_unsigned_input(),
            swap_box_2.as_unsigned_input(),
        ]
        .try_into()
        .unwrap();

        let swap_box_1_owner_address = encoding::unwrap_string(&swap_box_1.registers()[0]).unwrap();
        let swap_box_1_value_after_fees: NanoErg = swap_box_1.nano_ergs() - (transaction_fee / 2);

        let swap_box_2_owner_address = encoding::unwrap_string(&swap_box_2.registers()[0]).unwrap();
        let swap_box_2_value_after_fees: NanoErg = swap_box_2.nano_ergs() - (transaction_fee / 2);

        let swap_box_1_fufilling_candidate = create_candidate(
            swap_box_1_value_after_fees,
            &swap_box_1_owner_address,
            &swap_box_2.tokens(),
            &vec![encoding::serialize_string(&swap_box_1_owner_address)],
            current_height,
        )
        .unwrap();

        let swap_box_2_fufilling_candidate = create_candidate(
            swap_box_2_value_after_fees,
            &swap_box_2_owner_address,
            &swap_box_1.tokens(),
            &vec![encoding::serialize_string(&swap_box_2_owner_address)],
            current_height,
        )
        .unwrap();

        let tx_fee_candidate = TxFeeBox::output_candidate(transaction_fee, current_height).unwrap();

        let output_candidates = vec![
            swap_box_1_fufilling_candidate,
            swap_box_2_fufilling_candidate,
            tx_fee_candidate,
        ]
        .try_into()
        .unwrap();

        UnsignedTransaction::new(tx_inputs, None, output_candidates).unwrap()
    }

    /// Returns a BoxSpec for a box that can fufill the given SwapBox
    pub fn get_swap_box_match_spec(swap_box: SwapBox) -> BoxSpec {
        // TODO: Deserialise order_token_id from SELF.R5[Coll[Byte]] into a &str
        let order_token_id = encoding::unwrap_string(&swap_box.registers()[1]).unwrap();
        let order_amount = encoding::unwrap_long(&swap_box.registers()[2]).unwrap() as u64;

        let reward_token_id: Vec<u8> = swap_box.tokens()[0].token_id.as_ref().try_into().unwrap();

        // Swap Box Match Spec
        BoxSpec::new(
            Some(SwapBox::CONTRACT_ADDRESS.to_string()),
            None,
            vec![
                RegisterSpec::new(Some(SType::SColl(Box::new(SType::SByte))), None),
                RegisterSpec::new(
                    Some(SType::SColl(Box::new(SType::SByte))),
                    Some(Constant::from(reward_token_id)),
                ),
                RegisterSpec::new(Some(SType::SLong), None),
            ],
            vec![Some(TokenSpec::new(
                order_amount..u64::MAX,
                &order_token_id,
            ))],
        )
    }
}
