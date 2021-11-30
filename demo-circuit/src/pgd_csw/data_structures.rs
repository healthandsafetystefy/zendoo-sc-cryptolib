use cctp_primitives::{
    type_mapping::FieldElement,
    utils::{data_structures::BackwardTransfer, get_bt_merkle_root},
};

use crate::{
    type_mapping::*, GingerMHTBinaryPath, MC_RETURN_ADDRESS_BYTES, PHANTOM_FIELD_ELEMENT,
    PHANTOM_PUBLIC_KEY_BITS, PHANTOM_SECRET_KEY_BITS,
};

#[derive(Clone)]
pub struct WithdrawalCertificateData {
    pub(crate) ledger_id: FieldElement,
    pub(crate) epoch_id: u32,
    pub(crate) bt_root: FieldElement, // Merkle root hash of all BTs from the certificate (recall that MC hashes all complex proof_data params from the certificate)
    pub(crate) quality: u64,
    pub(crate) mcb_sc_txs_com: FieldElement,
    pub(crate) ft_min_fee: u64,
    pub(crate) btr_min_fee: u64,
    pub(crate) custom_fields: Vec<FieldElement>,
}

impl WithdrawalCertificateData {
    pub fn new(
        ledger_id: FieldElement,
        epoch_id: u32,
        bt_list: Vec<BackwardTransfer>,
        quality: u64,
        mcb_sc_txs_com: FieldElement,
        ft_min_fee: u64,
        btr_min_fee: u64,
        custom_fields: Vec<FieldElement>,
    ) -> Self {
        let bt_root = get_bt_merkle_root(if !bt_list.is_empty() {
            Some(&bt_list)
        } else {
            None
        })
        .unwrap();
        Self {
            ledger_id,
            epoch_id,
            bt_root,
            quality,
            mcb_sc_txs_com,
            ft_min_fee,
            btr_min_fee,
            custom_fields,
        }
    }

    pub fn get_phantom_data(num_custom_fields: usize) -> Self {
        Self {
            ledger_id: PHANTOM_FIELD_ELEMENT,
            epoch_id: 0,
            bt_root: get_bt_merkle_root(None).unwrap(),
            quality: 0,
            mcb_sc_txs_com: PHANTOM_FIELD_ELEMENT,
            ft_min_fee: 0,
            btr_min_fee: 0,
            custom_fields: vec![PHANTOM_FIELD_ELEMENT; num_custom_fields],
        }
    }
}

#[derive(Clone)]
pub struct CswUtxoOutputData {
    pub spending_pub_key: [bool; SIMULATED_FIELD_BYTE_SIZE * 8],
    pub amount: u64,
    pub nonce: u64,
    pub custom_hash: [bool; FIELD_SIZE * 8],
}

impl Default for CswUtxoOutputData {
    fn default() -> Self {
        Self {
            spending_pub_key: PHANTOM_PUBLIC_KEY_BITS,
            amount: 0,
            nonce: 0,
            custom_hash: [false; FIELD_SIZE * 8],
        }
    }
}

// TODO: is it ok to consider "phantom" the default instance of this struct?
#[derive(Clone)]
pub struct CswUtxoInputData {
    pub output: CswUtxoOutputData,
    pub secret_key: [bool; SIMULATED_SCALAR_FIELD_MODULUS_BITS],
}

impl Default for CswUtxoInputData {
    fn default() -> Self {
        Self {
            output: CswUtxoOutputData::default(),
            secret_key: PHANTOM_SECRET_KEY_BITS,
        }
    }
}

// TODO: is it ok to consider "phantom" the default instance of this struct?
#[derive(Clone)]
pub struct CswFtInputData {
    pub amount: u64,
    pub receiver_pub_key: [bool; SIMULATED_FIELD_BYTE_SIZE * 8],
    pub payback_addr_data_hash: [bool; MC_RETURN_ADDRESS_BYTES * 8],
    pub tx_hash: [bool; FIELD_SIZE * 8],
    pub out_idx: u32,
}

impl Default for CswFtInputData {
    fn default() -> Self {
        Self {
            amount: 0,
            receiver_pub_key: PHANTOM_PUBLIC_KEY_BITS,
            payback_addr_data_hash: [false; MC_RETURN_ADDRESS_BYTES * 8],
            tx_hash: [false; FIELD_SIZE * 8],
            out_idx: 0,
        }
    }
}

#[derive(Clone)]
pub struct CswProverData {
    // public inputs [START]

    // sys_data [START]
    pub genesis_constant: FieldElement, // Passed directly by the MC. It is a constant declared during SC creation that commits to various SC params. In the current SNARK design it isn't used (but might be usefull for other sidechains), so just ignored in the circuit. Note that it is the same constant as for WCert proof.
    pub mcb_sc_txs_com_end: FieldElement, // Passed directly by MC. The cumulative SCTxsCommitment hash taken from the MC block where the SC was ceased (needed to recover FTs in reverted epochs).
    pub sc_last_wcert_hash: FieldElement, // hash of the last confirmed WCert (excluding reverted) for this sidechain (calculated directly by MC). Note that it should be a hash of WithdrawalCertificateData
    pub amount: FieldElement,             // taken from CSW and passed directly by the MC
    pub nullifier: FieldElement,          // taken from CSW and passed directly by the MC
    pub receiver: FieldElement, // the receiver is fixed by the proof, otherwise someone will be able to front-run the tx and steel the proof. Note that we actually don't need to do anything with the receiver in the circuit, it's enough just to have it as a public input
    // sys_data [END]

    // public inputs [END]

    // witnesses [START]
    pub last_wcert: WithdrawalCertificateData, // the last confirmed wcert in the MC, it can be NULL (phantom)

    // either `input` or `ft_input` must be non NULL
    pub input: CswUtxoInputData, // unspent output we are trying to withdraw
    pub mst_path_to_output: GingerMHTBinaryPath, // path to output in the MST of the known state
    pub ft_input: CswFtInputData, // FT output in the MC block
    pub ft_input_secret_key: [bool; SIMULATED_SCALAR_FIELD_MODULUS_BITS], // secret key that authorizes ft_input spending
    pub mcb_sc_txs_com_start: FieldElement, // Cumulative ScTxsCommittment taken from the last MC block of the last confirmed (not reverted) epoch
    pub merkle_path_to_sc_hash: GingerMHTBinaryPath, // Merkle path to a particular sidechain in the ScTxsComm tree
    pub ft_tree_path: GingerMHTBinaryPath, // path to the ft_input_hash in the FT Merkle tree included in ScTxsComm tree
    pub scb_btr_tree_root: FieldElement,   // root hash of the BTR tree included in ScTxsComm tree
    pub wcert_tree_root: FieldElement,     // root hash of the Wcert tree included in ScTxsComm tree
    pub sc_txs_com_hashes: Vec<FieldElement>, // contains all ScTxsComm cumulative hashes on the way from `mcb_sc_txs_com_start` to `mcb_sc_txs_com_end`
                                              // RANGE_SIZE is a number of blocks between `mcb_sc_txs_com_start` and `mcb_sc_txs_com_end`.
                                              // It seems it can be a constant as the number of blocks between the last confirmed block and SC ceasing block should be fixed for a particular sidechain
                                              // witnesses [END]
}
