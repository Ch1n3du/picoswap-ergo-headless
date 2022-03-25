use ergo_headless_dapp_framework::*;
use crate::parsing;
pub use ergo_lib::*;

/*
======================================================================
SWAPBOX ERGOSCRIPT
======================================================================
    * @notice This box holds a token to be swapped for an arbitrary token that isn't Erg or can be refunded
    *         to orderOwner.
    * @param orderOwner Address to which is in control of the swap box (Base58 encoded string).
    * @param orderTokenId TokenId for token to be payed to orderOwner(Base16 encoded string).
    * @param orderAmount Amount of orderTokenId to be payed to orderOwner (Long).
    {
        val defined = {
	        SELF.R4[SigmaProp].isDefined && 
	        SELF.R5[(Coll[Byte], Long)].isDefined &&
            SELF.R6[Long].isDefined
        }

        val orderOwner = SELF.R4[SigmaProp].get
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

#[derive(Debug, Clone, WrapBox, SpecBox)]
pub struct SwapBox {
    ergo_box: ErgoBox,
}

impl SwapBox {
    const CONTRACT_ADDRESS: &'static str = "THIS IS RUBBISH";
}

impl SpecifiedBox for SwapBox {
    fn box_spec() -> BoxSpec {
        let owner_address = RegisterSpec::new(
            Some(SType::SColl(Box::new(SType::SByte))),
            None
        );
        let order_token_id = RegisterSpec::new(
            Some(SType::SColl(Box::new(SType::SByte))),
            None,
        );
        let order_token_amount = RegisterSpec::new(
            Some(SType::SLong),
            None,
        );

        BoxSpec::new(Some(Self::CONTRACT_ADDRESS.to_string()),
            None,
            vec![
                owner_address,
                order_token_id,
                order_token_amount,
            ], 
            vec![])
    }
}

pub struct SwapProtocol {}

impl SwapProtocol {
    

    // @notice Action to create SwapBox instance.
    pub fn action_create_swap_box(
        _order_owner: String,
        _order_token_id: String,
        _order_amount: u64,
        ergs_box_for_swap: ErgsBox,
        current_height: u64,
        transaction_fee: u64,
    ) -> UnsignedTransaction {

        let tx_inputs = vec![
            ergs_box_for_swap.as_unsigned_input(),
        ].try_into().unwrap();

        let swap_reward = ergs_box_for_swap.tokens()[0].clone();
        let value_after_fees = ergs_box_for_swap.nano_ergs() - transaction_fee;

        let order_owner = parsing::string_to_constant(_order_owner);
        let order_token_id = parsing::string_to_constant(_order_token_id);
        let order_amount = Constant::from(_order_amount as i64);

        let swap_box_candidate = create_candidate(
            value_after_fees, 
            &SwapBox::CONTRACT_ADDRESS.to_string(), 
            &vec![swap_reward], 
            &vec![order_owner, order_token_id, order_amount], 
            current_height
        ).unwrap();

        let transaction_fee_candidate =
           TxFeeBox::output_candidate(transaction_fee, current_height).unwrap();

        let output_candidates = vec![
            swap_box_candidate,
            transaction_fee_candidate
        ].try_into().unwrap();
    
        UnsignedTransaction::new(tx_inputs, None, output_candidates).unwrap()
    }

    // @notice Action to reclaim SwapBox instance by orderOwner :(
    pub fn action_reclaim_swap(
        order_owner: String,
        swap_box_to_reclaim: SwapBox,
        current_height: u64,
        transaction_fee: u64,
    ) -> UnsignedTransaction {
        let tx_inputs = vec![
            swap_box_to_reclaim.as_unsigned_input(),
        ].try_into().unwrap();

        let refund_value_after_fees = swap_box_to_reclaim.nano_ergs() - transaction_fee;
        let swap_token = swap_box_to_reclaim.tokens()[0].clone();

        let refund_candidate = create_candidate(
            refund_value_after_fees, 
            &order_owner, 
            &vec![swap_token], 
            &vec![], 
            current_height
        ).unwrap();

        let transaction_fee_candidate = 
            TxFeeBox::output_candidate(transaction_fee, current_height).unwrap();

        let output_candidates = vec![
            refund_candidate,
            transaction_fee_candidate
        ].try_into().unwrap();

        UnsignedTransaction::new(tx_inputs, None, output_candidates).unwrap()
    }

    // @notice Takes two boxes that can fufill each other and execute swap :) 
    // @param executor_address Address to collect fees and change
    pub fn action_execute_swap(
        swap_box: SwapBox,
        swap_owner_address: String,
        ergs_box_to_fufill: ErgsBox,
        fufiller_address: String,
        current_height: u64,
        transaction_fee: u64,
    ) -> UnsignedTransaction {
        let tx_inputs = vec![
            swap_box.as_unsigned_input(),
            ergs_box_to_fufill.as_unsigned_input(),
        ].try_into().unwrap();

        let value_after_fee = swap_box.nano_ergs() - transaction_fee;
        let swap_fufilling_candidate = create_candidate(
            value_after_fee, 
            &swap_owner_address, 
            &vec![ergs_box_to_fufill.tokens()[0].clone()], 
            &vec![], 
            current_height
        ).unwrap();
        
        let tx_fee_candidate = TxFeeBox::output_candidate(
            transaction_fee, 
            current_height
        ).unwrap();

        let reward_candidate = ChangeBox::output_candidate(
            &vec![swap_box.tokens()[0].clone()], 
           ergs_box_to_fufill.nano_ergs(), 
            &fufiller_address,
            current_height
        ).unwrap();

        let output_candidates = vec![
            tx_fee_candidate,
            swap_fufilling_candidate,
            reward_candidate,
        ].try_into().unwrap();

        UnsignedTransaction::new(tx_inputs, None, output_candidates).unwrap()
    }

    // @notice Gets the swap requirements of box and fetches an ErgsBox matching said requirements.
    pub fn match_swap_box(
        swap_box: SwapBox,
        explorer_endpoint: String
    ) -> ErgsBox {
        let order_owner = parsing::constant_to_base_58_str(swap_box.registers()[0].clone());
        let order_token_id = parsing::constant_to_base_16_str(swap_box.registers()[1].clone());
        let order_amount = parsing::constant_to_u64(swap_box.registers()[2].clone());

        let reward_token_id = swap_box.tokens()[0]
            .clone()
            .token_id;
        let reward_token_amount = swap_box.tokens()[0]
            .clone()
            .amount;

        let swap_match_spec = ErgsBox::box_spec()
            .modified_address(Some(SwapBox::CONTRACT_ADDRESS.to_string()))
            .modified_tokens(vec![
                Some(TokenSpec::new(order_amount..u64::MAX, order_token_id)),
            ])
            .modified_registers(vec![
                RegisterSpec::new(Some(SType::SColl(Box::new(SType::SByte))), None),
                RegisterSpec::new(Some(SType::SColl(Box::new(SType::SByte))), None),
                RegisterSpec::new(Some(SType::SLong), None),
            ]);

        todo!()
    }
}