//! Payload types.

use crate::PayloadError;
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use alloy_consensus::{Blob, Bytes48};
use alloy_eips::{eip4844::BlobTransactionSidecar, eip4895::Withdrawal, BlockNumHash};
use alloy_primitives::{Address, Bloom, Bytes, B256, B64, U256};
use core::iter::{FromIterator, IntoIterator};

/// The execution payload body response that allows for `null` values.
pub type ExecutionPayloadBodiesV1 = Vec<Option<ExecutionPayloadBodyV1>>;

/// And 8-byte identifier for an execution payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PayloadId(pub B64);

// === impl PayloadId ===

impl PayloadId {
    /// Creates a new payload id from the given identifier.
    pub fn new(id: [u8; 8]) -> Self {
        Self(B64::from(id))
    }
}

impl core::fmt::Display for PayloadId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

/// This represents the `executionPayload` field in the return value of `engine_getPayloadV2`,
/// specified as:
///
/// - `executionPayload`: `ExecutionPayloadV1` | `ExecutionPayloadV2` where:
///   - `ExecutionPayloadV1` **MUST** be returned if the payload `timestamp` is lower than the
///     Shanghai timestamp
///   - `ExecutionPayloadV2` **MUST** be returned if the payload `timestamp` is greater or equal to
///     the Shanghai timestamp
///
/// See:
/// <https://github.com/ethereum/execution-apis/blob/fe8e13c288c592ec154ce25c534e26cb7ce0530d/src/engine/shanghai.md#response>
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum ExecutionPayloadFieldV2 {
    /// V1 payload
    V1(ExecutionPayloadV1),
    /// V2 payload
    V2(ExecutionPayloadV2),
}

impl ExecutionPayloadFieldV2 {
    /// Returns the inner [ExecutionPayloadV1]
    pub fn into_v1_payload(self) -> ExecutionPayloadV1 {
        match self {
            Self::V1(payload) => payload,
            Self::V2(payload) => payload.payload_inner,
        }
    }
}

/// This is the input to `engine_newPayloadV2`, which may or may not have a withdrawals field.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase", deny_unknown_fields))]
pub struct ExecutionPayloadInputV2 {
    /// The V1 execution payload
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub execution_payload: ExecutionPayloadV1,
    /// The payload withdrawals
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub withdrawals: Option<Vec<Withdrawal>>,
}

/// This structure maps for the return value of `engine_getPayload` of the beacon chain spec, for
/// V2.
///
/// See also:
/// <https://github.com/ethereum/execution-apis/blob/main/src/engine/shanghai.md#engine_getpayloadv2>
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ExecutionPayloadEnvelopeV2 {
    /// Execution payload, which could be either V1 or V2
    ///
    /// V1 (_NO_ withdrawals) MUST be returned if the payload timestamp is lower than the Shanghai
    /// timestamp
    ///
    /// V2 (_WITH_ withdrawals) MUST be returned if the payload timestamp is greater or equal to
    /// the Shanghai timestamp
    pub execution_payload: ExecutionPayloadFieldV2,
    /// The expected value to be received by the feeRecipient in wei
    pub block_value: U256,
}

impl ExecutionPayloadEnvelopeV2 {
    /// Returns the [ExecutionPayload] for the `engine_getPayloadV1` endpoint
    pub fn into_v1_payload(self) -> ExecutionPayloadV1 {
        self.execution_payload.into_v1_payload()
    }
}

/// This structure maps for the return value of `engine_getPayload` of the beacon chain spec, for
/// V3.
///
/// See also:
/// <https://github.com/ethereum/execution-apis/blob/fe8e13c288c592ec154ce25c534e26cb7ce0530d/src/engine/cancun.md#response-2>
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ExecutionPayloadEnvelopeV3 {
    /// Execution payload V3
    pub execution_payload: ExecutionPayloadV3,
    /// The expected value to be received by the feeRecipient in wei
    pub block_value: U256,
    /// The blobs, commitments, and proofs associated with the executed payload.
    pub blobs_bundle: BlobsBundleV1,
    /// Introduced in V3, this represents a suggestion from the execution layer if the payload
    /// should be used instead of an externally provided one.
    pub should_override_builder: bool,
}

/// This structure maps for the return value of `engine_getPayload` of the beacon chain spec, for
/// V4.
///
/// See also:
/// <https://github.com/ethereum/execution-apis/blob/main/src/engine/prague.md#engine_getpayloadv4>
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ExecutionPayloadEnvelopeV4 {
    /// Execution payload V3
    pub execution_payload: ExecutionPayloadV3,
    /// The expected value to be received by the feeRecipient in wei
    pub block_value: U256,
    /// The blobs, commitments, and proofs associated with the executed payload.
    pub blobs_bundle: BlobsBundleV1,
    /// Introduced in V3, this represents a suggestion from the execution layer if the payload
    /// should be used instead of an externally provided one.
    pub should_override_builder: bool,
    /// A list of opaque [EIP-7685][eip7685] requests.
    ///
    /// [eip7685]: https://eips.ethereum.org/EIPS/eip-7685
    pub execution_requests: Vec<Bytes>,
}

/// This structure maps on the ExecutionPayload structure of the beacon chain spec.
///
/// See also: <https://github.com/ethereum/execution-apis/blob/6709c2a795b707202e93c4f2867fa0bf2640a84f/src/engine/paris.md#executionpayloadv1>
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "ssz", derive(ssz_derive::Encode, ssz_derive::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ExecutionPayloadV1 {
    /// The parent hash of the block.
    pub parent_hash: B256,
    /// The fee recipient of the block.
    pub fee_recipient: Address,
    /// The state root of the block.
    pub state_root: B256,
    /// The receipts root of the block.
    pub receipts_root: B256,
    /// The logs bloom of the block.
    pub logs_bloom: Bloom,
    /// The previous randao of the block.
    pub prev_randao: B256,
    /// The block number.
    #[cfg_attr(feature = "serde", serde(with = "alloy_serde::quantity"))]
    pub block_number: u64,
    /// The gas limit of the block.
    #[cfg_attr(feature = "serde", serde(with = "alloy_serde::quantity"))]
    pub gas_limit: u64,
    /// The gas used of the block.
    #[cfg_attr(feature = "serde", serde(with = "alloy_serde::quantity"))]
    pub gas_used: u64,
    /// The timestamp of the block.
    #[cfg_attr(feature = "serde", serde(with = "alloy_serde::quantity"))]
    pub timestamp: u64,
    /// The extra data of the block.
    pub extra_data: Bytes,
    /// The base fee per gas of the block.
    pub base_fee_per_gas: U256,
    /// The block hash of the block.
    pub block_hash: B256,
    /// The transactions of the block.
    pub transactions: Vec<Bytes>,
    /// difficulty for N42
    pub difficulty: U256,
    /// nonce for N42
    pub nonce: B64,
}

impl ExecutionPayloadV1 {
    /// Returns the block number and hash as a [`BlockNumHash`].
    pub const fn block_num_hash(&self) -> BlockNumHash {
        BlockNumHash::new(self.block_number, self.block_hash)
    }
}

/// This structure maps on the ExecutionPayloadV2 structure of the beacon chain spec.
///
/// See also: <https://github.com/ethereum/execution-apis/blob/6709c2a795b707202e93c4f2867fa0bf2640a84f/src/engine/shanghai.md#executionpayloadv2>
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase", deny_unknown_fields))]
pub struct ExecutionPayloadV2 {
    /// Inner V1 payload
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub payload_inner: ExecutionPayloadV1,

    /// Array of [`Withdrawal`] enabled with V2
    /// See <https://github.com/ethereum/execution-apis/blob/6709c2a795b707202e93c4f2867fa0bf2640a84f/src/engine/shanghai.md#executionpayloadv2>
    pub withdrawals: Vec<Withdrawal>,
}

impl ExecutionPayloadV2 {
    /// Returns the timestamp for the execution payload.
    pub const fn timestamp(&self) -> u64 {
        self.payload_inner.timestamp
    }
}

#[cfg(feature = "ssz")]
impl ssz::Decode for ExecutionPayloadV2 {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let mut builder = ssz::SszDecoderBuilder::new(bytes);

        builder.register_type::<B256>()?;
        builder.register_type::<Address>()?;
        builder.register_type::<B256>()?;
        builder.register_type::<B256>()?;
        builder.register_type::<Bloom>()?;
        builder.register_type::<B256>()?;
        builder.register_type::<u64>()?;
        builder.register_type::<u64>()?;
        builder.register_type::<u64>()?;
        builder.register_type::<u64>()?;
        builder.register_type::<Bytes>()?;
        builder.register_type::<U256>()?;
        builder.register_type::<B256>()?;
        builder.register_type::<Vec<Bytes>>()?;
        builder.register_type::<Vec<Withdrawal>>()?;

        let mut decoder = builder.build()?;

        Ok(Self {
            payload_inner: ExecutionPayloadV1 {
                parent_hash: decoder.decode_next()?,
                fee_recipient: decoder.decode_next()?,
                state_root: decoder.decode_next()?,
                receipts_root: decoder.decode_next()?,
                logs_bloom: decoder.decode_next()?,
                prev_randao: decoder.decode_next()?,
                block_number: decoder.decode_next()?,
                gas_limit: decoder.decode_next()?,
                gas_used: decoder.decode_next()?,
                timestamp: decoder.decode_next()?,
                extra_data: decoder.decode_next()?,
                base_fee_per_gas: decoder.decode_next()?,
                block_hash: decoder.decode_next()?,
                transactions: decoder.decode_next()?,
                difficulty: decoder.decode_next()?,
                nonce: decoder.decode_next()?,
            },
            withdrawals: decoder.decode_next()?,
        })
    }
}

#[cfg(feature = "ssz")]
impl ssz::Encode for ExecutionPayloadV2 {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        let offset = <B256 as ssz::Encode>::ssz_fixed_len() * 5
            + <Address as ssz::Encode>::ssz_fixed_len()
            + <Bloom as ssz::Encode>::ssz_fixed_len()
            + <u64 as ssz::Encode>::ssz_fixed_len() * 4
            + <U256 as ssz::Encode>::ssz_fixed_len()
            + ssz::BYTES_PER_LENGTH_OFFSET * 3;

        let mut encoder = ssz::SszEncoder::container(buf, offset);

        encoder.append(&self.payload_inner.parent_hash);
        encoder.append(&self.payload_inner.fee_recipient);
        encoder.append(&self.payload_inner.state_root);
        encoder.append(&self.payload_inner.receipts_root);
        encoder.append(&self.payload_inner.logs_bloom);
        encoder.append(&self.payload_inner.prev_randao);
        encoder.append(&self.payload_inner.block_number);
        encoder.append(&self.payload_inner.gas_limit);
        encoder.append(&self.payload_inner.gas_used);
        encoder.append(&self.payload_inner.timestamp);
        encoder.append(&self.payload_inner.extra_data);
        encoder.append(&self.payload_inner.base_fee_per_gas);
        encoder.append(&self.payload_inner.block_hash);
        encoder.append(&self.payload_inner.transactions);
        encoder.append(&self.withdrawals);

        encoder.finalize();
    }

    fn ssz_bytes_len(&self) -> usize {
        <ExecutionPayloadV1 as ssz::Encode>::ssz_bytes_len(&self.payload_inner)
            + ssz::BYTES_PER_LENGTH_OFFSET
            + self.withdrawals.ssz_bytes_len()
    }
}

/// This structure maps on the ExecutionPayloadV3 structure of the beacon chain spec.
///
/// See also: <https://github.com/ethereum/execution-apis/blob/6709c2a795b707202e93c4f2867fa0bf2640a84f/src/engine/shanghai.md#executionpayloadv2>
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ExecutionPayloadV3 {
    /// Inner V2 payload
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub payload_inner: ExecutionPayloadV2,

    /// Array of hex [`u64`] representing blob gas used, enabled with V3
    /// See <https://github.com/ethereum/execution-apis/blob/fe8e13c288c592ec154ce25c534e26cb7ce0530d/src/engine/cancun.md#ExecutionPayloadV3>
    #[cfg_attr(feature = "serde", serde(with = "alloy_serde::quantity"))]
    pub blob_gas_used: u64,
    /// Array of hex[`u64`] representing excess blob gas, enabled with V3
    /// See <https://github.com/ethereum/execution-apis/blob/fe8e13c288c592ec154ce25c534e26cb7ce0530d/src/engine/cancun.md#ExecutionPayloadV3>
    #[cfg_attr(feature = "serde", serde(with = "alloy_serde::quantity"))]
    pub excess_blob_gas: u64,
}

impl ExecutionPayloadV3 {
    /// Returns the withdrawals for the payload.
    pub const fn withdrawals(&self) -> &Vec<Withdrawal> {
        &self.payload_inner.withdrawals
    }

    /// Returns the timestamp for the payload.
    pub const fn timestamp(&self) -> u64 {
        self.payload_inner.payload_inner.timestamp
    }
}

#[cfg(feature = "ssz")]
impl ssz::Decode for ExecutionPayloadV3 {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let mut builder = ssz::SszDecoderBuilder::new(bytes);

        builder.register_type::<B256>()?;
        builder.register_type::<Address>()?;
        builder.register_type::<B256>()?;
        builder.register_type::<B256>()?;
        builder.register_type::<Bloom>()?;
        builder.register_type::<B256>()?;
        builder.register_type::<u64>()?;
        builder.register_type::<u64>()?;
        builder.register_type::<u64>()?;
        builder.register_type::<u64>()?;
        builder.register_type::<Bytes>()?;
        builder.register_type::<U256>()?;
        builder.register_type::<B256>()?;
        builder.register_type::<Vec<Bytes>>()?;
        builder.register_type::<Vec<Withdrawal>>()?;
        builder.register_type::<u64>()?;
        builder.register_type::<u64>()?;

        let mut decoder = builder.build()?;

        Ok(Self {
            payload_inner: ExecutionPayloadV2 {
                payload_inner: ExecutionPayloadV1 {
                    parent_hash: decoder.decode_next()?,
                    fee_recipient: decoder.decode_next()?,
                    state_root: decoder.decode_next()?,
                    receipts_root: decoder.decode_next()?,
                    logs_bloom: decoder.decode_next()?,
                    prev_randao: decoder.decode_next()?,
                    block_number: decoder.decode_next()?,
                    gas_limit: decoder.decode_next()?,
                    gas_used: decoder.decode_next()?,
                    timestamp: decoder.decode_next()?,
                    extra_data: decoder.decode_next()?,
                    base_fee_per_gas: decoder.decode_next()?,
                    block_hash: decoder.decode_next()?,
                    transactions: decoder.decode_next()?,
                    difficulty: decoder.decode_next()?,
                    nonce: decoder.decode_next()?,
                },
                withdrawals: decoder.decode_next()?,
            },
            blob_gas_used: decoder.decode_next()?,
            excess_blob_gas: decoder.decode_next()?,
        })
    }
}

#[cfg(feature = "ssz")]
impl ssz::Encode for ExecutionPayloadV3 {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        let offset = <B256 as ssz::Encode>::ssz_fixed_len() * 5
            + <Address as ssz::Encode>::ssz_fixed_len()
            + <Bloom as ssz::Encode>::ssz_fixed_len()
            + <u64 as ssz::Encode>::ssz_fixed_len() * 6
            + <U256 as ssz::Encode>::ssz_fixed_len()
            + ssz::BYTES_PER_LENGTH_OFFSET * 3;

        let mut encoder = ssz::SszEncoder::container(buf, offset);

        encoder.append(&self.payload_inner.payload_inner.parent_hash);
        encoder.append(&self.payload_inner.payload_inner.fee_recipient);
        encoder.append(&self.payload_inner.payload_inner.state_root);
        encoder.append(&self.payload_inner.payload_inner.receipts_root);
        encoder.append(&self.payload_inner.payload_inner.logs_bloom);
        encoder.append(&self.payload_inner.payload_inner.prev_randao);
        encoder.append(&self.payload_inner.payload_inner.block_number);
        encoder.append(&self.payload_inner.payload_inner.gas_limit);
        encoder.append(&self.payload_inner.payload_inner.gas_used);
        encoder.append(&self.payload_inner.payload_inner.timestamp);
        encoder.append(&self.payload_inner.payload_inner.extra_data);
        encoder.append(&self.payload_inner.payload_inner.base_fee_per_gas);
        encoder.append(&self.payload_inner.payload_inner.block_hash);
        encoder.append(&self.payload_inner.payload_inner.transactions);
        encoder.append(&self.payload_inner.withdrawals);
        encoder.append(&self.blob_gas_used);
        encoder.append(&self.excess_blob_gas);

        encoder.finalize();
    }

    fn ssz_bytes_len(&self) -> usize {
        <ExecutionPayloadV2 as ssz::Encode>::ssz_bytes_len(&self.payload_inner)
            + <u64 as ssz::Encode>::ssz_fixed_len() * 2
    }
}

/// This includes all bundled blob related data of an executed payload.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlobsBundleV1 {
    /// All commitments in the bundle.
    pub commitments: Vec<alloy_consensus::Bytes48>,
    /// All proofs in the bundle.
    pub proofs: Vec<alloy_consensus::Bytes48>,
    /// All blobs in the bundle.
    pub blobs: Vec<alloy_consensus::Blob>,
}

#[cfg(feature = "ssz")]
#[derive(ssz_derive::Encode, ssz_derive::Decode)]
struct BlobsBundleV1Ssz {
    commitments: Vec<alloy_primitives::FixedBytes<48>>,
    proofs: Vec<alloy_primitives::FixedBytes<48>>,
    blobs: Vec<alloy_primitives::FixedBytes<{ alloy_eips::eip4844::BYTES_PER_BLOB }>>,
}

#[cfg(feature = "ssz")]
impl BlobsBundleV1Ssz {
    const _ASSERT: [(); std::mem::size_of::<BlobsBundleV1>()] = [(); std::mem::size_of::<Self>()];

    const fn wrap_ref(other: &BlobsBundleV1) -> &Self {
        // SAFETY: Same repr and size
        unsafe { &*(other as *const BlobsBundleV1 as *const Self) }
    }

    fn unwrap(self) -> BlobsBundleV1 {
        // SAFETY: Same repr and size
        unsafe { std::mem::transmute(self) }
    }
}

#[cfg(feature = "ssz")]
impl ssz::Encode for BlobsBundleV1 {
    fn is_ssz_fixed_len() -> bool {
        <BlobsBundleV1Ssz as ssz::Encode>::is_ssz_fixed_len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        BlobsBundleV1Ssz::wrap_ref(self).ssz_append(buf)
    }

    fn ssz_fixed_len() -> usize {
        <BlobsBundleV1Ssz as ssz::Encode>::ssz_fixed_len()
    }

    fn ssz_bytes_len(&self) -> usize {
        BlobsBundleV1Ssz::wrap_ref(self).ssz_bytes_len()
    }

    fn as_ssz_bytes(&self) -> Vec<u8> {
        BlobsBundleV1Ssz::wrap_ref(self).as_ssz_bytes()
    }
}

#[cfg(feature = "ssz")]
impl ssz::Decode for BlobsBundleV1 {
    fn is_ssz_fixed_len() -> bool {
        <BlobsBundleV1Ssz as ssz::Decode>::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        <BlobsBundleV1Ssz as ssz::Decode>::ssz_fixed_len()
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        BlobsBundleV1Ssz::from_ssz_bytes(bytes).map(BlobsBundleV1Ssz::unwrap)
    }
}

impl BlobsBundleV1 {
    /// Creates a new blob bundle from the given sidecars.
    ///
    /// This folds the sidecar fields into single commit, proof, and blob vectors.
    pub fn new(sidecars: impl IntoIterator<Item = BlobTransactionSidecar>) -> Self {
        let (commitments, proofs, blobs) = sidecars.into_iter().fold(
            (Vec::new(), Vec::new(), Vec::new()),
            |(mut commitments, mut proofs, mut blobs), sidecar| {
                commitments.extend(sidecar.commitments);
                proofs.extend(sidecar.proofs);
                blobs.extend(sidecar.blobs);
                (commitments, proofs, blobs)
            },
        );
        Self { commitments, proofs, blobs }
    }

    /// Take `len` blob data from the bundle.
    ///
    /// # Panics
    ///
    /// If len is more than the blobs bundle len.
    pub fn take(&mut self, len: usize) -> (Vec<Bytes48>, Vec<Bytes48>, Vec<Blob>) {
        (
            self.commitments.drain(0..len).collect(),
            self.proofs.drain(0..len).collect(),
            self.blobs.drain(0..len).collect(),
        )
    }

    /// Returns the sidecar from the bundle
    ///
    /// # Panics
    ///
    /// If len is more than the blobs bundle len.
    pub fn pop_sidecar(&mut self, len: usize) -> BlobTransactionSidecar {
        let (commitments, proofs, blobs) = self.take(len);
        BlobTransactionSidecar { commitments, proofs, blobs }
    }
}

impl From<Vec<BlobTransactionSidecar>> for BlobsBundleV1 {
    fn from(sidecars: Vec<BlobTransactionSidecar>) -> Self {
        Self::new(sidecars)
    }
}

impl FromIterator<BlobTransactionSidecar> for BlobsBundleV1 {
    fn from_iter<T: IntoIterator<Item = BlobTransactionSidecar>>(iter: T) -> Self {
        Self::new(iter)
    }
}

/// An execution payload, which can be either [ExecutionPayloadV1], [ExecutionPayloadV2], or
/// [ExecutionPayloadV3].
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum ExecutionPayload {
    /// V1 payload
    V1(ExecutionPayloadV1),
    /// V2 payload
    V2(ExecutionPayloadV2),
    /// V3 payload
    V3(ExecutionPayloadV3),
}

impl ExecutionPayload {
    /// Returns a reference to the V1 payload.
    pub const fn as_v1(&self) -> &ExecutionPayloadV1 {
        match self {
            Self::V1(payload) => payload,
            Self::V2(payload) => &payload.payload_inner,
            Self::V3(payload) => &payload.payload_inner.payload_inner,
        }
    }

    /// Returns a mutable reference to the V1 payload.
    pub fn as_v1_mut(&mut self) -> &mut ExecutionPayloadV1 {
        match self {
            Self::V1(payload) => payload,
            Self::V2(payload) => &mut payload.payload_inner,
            Self::V3(payload) => &mut payload.payload_inner.payload_inner,
        }
    }

    /// Consumes the payload and returns the V1 payload.
    pub fn into_v1(self) -> ExecutionPayloadV1 {
        match self {
            Self::V1(payload) => payload,
            Self::V2(payload) => payload.payload_inner,
            Self::V3(payload) => payload.payload_inner.payload_inner,
        }
    }

    /// Returns a reference to the V2 payload, if any.
    pub const fn as_v2(&self) -> Option<&ExecutionPayloadV2> {
        match self {
            Self::V1(_) => None,
            Self::V2(payload) => Some(payload),
            Self::V3(payload) => Some(&payload.payload_inner),
        }
    }

    /// Returns a mutable reference to the V2 payload, if any.
    pub fn as_v2_mut(&mut self) -> Option<&mut ExecutionPayloadV2> {
        match self {
            Self::V1(_) => None,
            Self::V2(payload) => Some(payload),
            Self::V3(payload) => Some(&mut payload.payload_inner),
        }
    }

    /// Returns a reference to the V2 payload, if any.
    pub const fn as_v3(&self) -> Option<&ExecutionPayloadV3> {
        match self {
            Self::V1(_) | Self::V2(_) => None,
            Self::V3(payload) => Some(payload),
        }
    }

    /// Returns a mutable reference to the V2 payload, if any.
    pub fn as_v3_mut(&mut self) -> Option<&mut ExecutionPayloadV3> {
        match self {
            Self::V1(_) | Self::V2(_) => None,
            Self::V3(payload) => Some(payload),
        }
    }

    /// Returns the withdrawals for the payload.
    pub const fn withdrawals(&self) -> Option<&Vec<Withdrawal>> {
        match self.as_v2() {
            Some(payload) => Some(&payload.withdrawals),
            None => None,
        }
    }

    /// Returns the timestamp for the payload.
    pub const fn timestamp(&self) -> u64 {
        self.as_v1().timestamp
    }

    /// Returns the parent hash for the payload.
    pub const fn parent_hash(&self) -> B256 {
        self.as_v1().parent_hash
    }

    /// Returns the block hash for the payload.
    pub const fn block_hash(&self) -> B256 {
        self.as_v1().block_hash
    }

    /// Returns the block number for this payload.
    pub const fn block_number(&self) -> u64 {
        self.as_v1().block_number
    }

    /// Returns the block number for this payload.
    pub const fn block_num_hash(&self) -> BlockNumHash {
        self.as_v1().block_num_hash()
    }

    /// Returns the prev randao for this payload.
    pub const fn prev_randao(&self) -> B256 {
        self.as_v1().prev_randao
    }
}

impl From<ExecutionPayloadV1> for ExecutionPayload {
    fn from(payload: ExecutionPayloadV1) -> Self {
        Self::V1(payload)
    }
}

impl From<ExecutionPayloadV2> for ExecutionPayload {
    fn from(payload: ExecutionPayloadV2) -> Self {
        Self::V2(payload)
    }
}

impl From<ExecutionPayloadV3> for ExecutionPayload {
    fn from(payload: ExecutionPayloadV3) -> Self {
        Self::V3(payload)
    }
}

// Deserializes untagged ExecutionPayload by trying each variant in falling order
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ExecutionPayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum ExecutionPayloadDesc {
            V3(ExecutionPayloadV3),
            V2(ExecutionPayloadV2),
            V1(ExecutionPayloadV1),
        }
        match ExecutionPayloadDesc::deserialize(deserializer)? {
            ExecutionPayloadDesc::V3(payload) => Ok(Self::V3(payload)),
            ExecutionPayloadDesc::V2(payload) => Ok(Self::V2(payload)),
            ExecutionPayloadDesc::V1(payload) => Ok(Self::V1(payload)),
        }
    }
}

/// This structure contains a body of an execution payload.
///
/// See also: <https://github.com/ethereum/execution-apis/blob/6452a6b194d7db269bf1dbd087a267251d3cc7f8/src/engine/shanghai.md#executionpayloadbodyv1>
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExecutionPayloadBodyV1 {
    /// Enveloped encoded transactions.
    pub transactions: Vec<Bytes>,
    /// All withdrawals in the block.
    ///
    /// Will always be `None` if pre shanghai.
    pub withdrawals: Option<Vec<Withdrawal>>,
}

/// This structure contains the attributes required to initiate a payload build process in the
/// context of an `engine_forkchoiceUpdated` call.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PayloadAttributes {
    /// Value for the `timestamp` field of the new payload
    #[cfg_attr(feature = "serde", serde(with = "alloy_serde::quantity"))]
    pub timestamp: u64,
    /// Value for the `prevRandao` field of the new payload
    pub prev_randao: B256,
    /// Suggested value for the `feeRecipient` field of the new payload
    pub suggested_fee_recipient: Address,
    /// Array of [`Withdrawal`] enabled with V2
    /// See <https://github.com/ethereum/execution-apis/blob/6452a6b194d7db269bf1dbd087a267251d3cc7f8/src/engine/shanghai.md#payloadattributesv2>
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub withdrawals: Option<Vec<Withdrawal>>,
    /// Root of the parent beacon block enabled with V3.
    ///
    /// See also <https://github.com/ethereum/execution-apis/blob/main/src/engine/cancun.md#payloadattributesv3>
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub parent_beacon_block_root: Option<B256>,
}

/// This structure contains the result of processing a payload or fork choice update.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PayloadStatus {
    /// The status of the payload.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub status: PayloadStatusEnum,
    /// Hash of the most recent valid block in the branch defined by payload and its ancestors
    pub latest_valid_hash: Option<B256>,
}

impl PayloadStatus {
    /// Initializes a new payload status.
    pub const fn new(status: PayloadStatusEnum, latest_valid_hash: Option<B256>) -> Self {
        Self { status, latest_valid_hash }
    }

    /// Creates a new payload status from the given status.
    pub const fn from_status(status: PayloadStatusEnum) -> Self {
        Self { status, latest_valid_hash: None }
    }

    /// Sets the latest valid hash.
    pub const fn with_latest_valid_hash(mut self, latest_valid_hash: B256) -> Self {
        self.latest_valid_hash = Some(latest_valid_hash);
        self
    }

    /// Sets the latest valid hash if it's not None.
    pub const fn maybe_latest_valid_hash(mut self, latest_valid_hash: Option<B256>) -> Self {
        self.latest_valid_hash = latest_valid_hash;
        self
    }

    /// Returns true if the payload status is syncing.
    pub const fn is_syncing(&self) -> bool {
        self.status.is_syncing()
    }

    /// Returns true if the payload status is valid.
    pub const fn is_valid(&self) -> bool {
        self.status.is_valid()
    }

    /// Returns true if the payload status is invalid.
    pub const fn is_invalid(&self) -> bool {
        self.status.is_invalid()
    }
}

impl core::fmt::Display for PayloadStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "PayloadStatus {{ status: {}, latestValidHash: {:?} }}",
            self.status, self.latest_valid_hash
        )
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for PayloadStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("status", self.status.as_str())?;
        map.serialize_entry("latestValidHash", &self.latest_valid_hash)?;
        map.serialize_entry("validationError", &self.status.validation_error())?;
        map.end()
    }
}

impl From<PayloadError> for PayloadStatusEnum {
    fn from(error: PayloadError) -> Self {
        Self::Invalid { validation_error: error.to_string() }
    }
}

/// Represents the status response of a payload.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE"))]
pub enum PayloadStatusEnum {
    /// VALID is returned by the engine API in the following calls:
    ///   - newPayload:       if the payload was already known or was just validated and executed
    ///   - forkchoiceUpdate: if the chain accepted the reorg (might ignore if it's stale)
    Valid,

    /// INVALID is returned by the engine API in the following calls:
    ///   - newPayload:       if the payload failed to execute on top of the local chain
    ///   - forkchoiceUpdate: if the new head is unknown, pre-merge, or reorg to it fails
    Invalid {
        /// The error message for the invalid payload.
        #[cfg_attr(feature = "serde", serde(rename = "validationError"))]
        validation_error: String,
    },

    /// SYNCING is returned by the engine API in the following calls:
    ///   - newPayload:       if the payload was accepted on top of an active sync
    ///   - forkchoiceUpdate: if the new head was seen before, but not part of the chain
    Syncing,

    /// ACCEPTED is returned by the engine API in the following calls:
    ///   - newPayload: if the payload was accepted, but not processed (side chain)
    Accepted,
}

impl PayloadStatusEnum {
    /// Returns the string representation of the payload status.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Valid => "VALID",
            Self::Invalid { .. } => "INVALID",
            Self::Syncing => "SYNCING",
            Self::Accepted => "ACCEPTED",
        }
    }

    /// Returns the validation error if the payload status is invalid.
    pub fn validation_error(&self) -> Option<&str> {
        match self {
            Self::Invalid { validation_error } => Some(validation_error),
            _ => None,
        }
    }

    /// Returns true if the payload status is syncing.
    pub const fn is_syncing(&self) -> bool {
        matches!(self, Self::Syncing)
    }

    /// Returns true if the payload status is valid.
    pub const fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    /// Returns true if the payload status is invalid.
    pub const fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid { .. })
    }
}

impl core::fmt::Display for PayloadStatusEnum {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Invalid { validation_error } => {
                f.write_str(self.as_str())?;
                f.write_str(": ")?;
                f.write_str(validation_error.as_str())
            }
            _ => f.write_str(self.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PayloadValidationError;
    use alloc::vec;
    use similar_asserts::assert_eq;

    #[test]
    #[cfg(feature = "serde")]
    fn serde_payload_status() {
        let s = r#"{"status":"SYNCING","latestValidHash":null,"validationError":null}"#;
        let status: PayloadStatus = serde_json::from_str(s).unwrap();
        assert_eq!(status.status, PayloadStatusEnum::Syncing);
        assert!(status.latest_valid_hash.is_none());
        assert!(status.status.validation_error().is_none());
        assert_eq!(serde_json::to_string(&status).unwrap(), s);

        let full = s;
        let s = r#"{"status":"SYNCING","latestValidHash":null}"#;
        let status: PayloadStatus = serde_json::from_str(s).unwrap();
        assert_eq!(status.status, PayloadStatusEnum::Syncing);
        assert!(status.latest_valid_hash.is_none());
        assert!(status.status.validation_error().is_none());
        assert_eq!(serde_json::to_string(&status).unwrap(), full);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_payload_status_error_deserialize() {
        let s = r#"{"status":"INVALID","latestValidHash":null,"validationError":"Failed to decode block"}"#;
        let q = PayloadStatus {
            latest_valid_hash: None,
            status: PayloadStatusEnum::Invalid {
                validation_error: "Failed to decode block".to_string(),
            },
        };
        assert_eq!(q, serde_json::from_str(s).unwrap());

        let s = r#"{"status":"INVALID","latestValidHash":null,"validationError":"links to previously rejected block"}"#;
        let q = PayloadStatus {
            latest_valid_hash: None,
            status: PayloadStatusEnum::Invalid {
                validation_error: PayloadValidationError::LinksToRejectedPayload.to_string(),
            },
        };
        assert_eq!(q, serde_json::from_str(s).unwrap());

        let s = r#"{"status":"INVALID","latestValidHash":null,"validationError":"invalid block number"}"#;
        let q = PayloadStatus {
            latest_valid_hash: None,
            status: PayloadStatusEnum::Invalid {
                validation_error: PayloadValidationError::InvalidBlockNumber.to_string(),
            },
        };
        assert_eq!(q, serde_json::from_str(s).unwrap());

        let s = r#"{"status":"INVALID","latestValidHash":null,"validationError":
        "invalid merkle root: (remote: 0x3f77fb29ce67436532fee970e1add8f5cc80e8878c79b967af53b1fd92a0cab7 local: 0x603b9628dabdaadb442a3bb3d7e0360efc110e1948472909230909f1690fed17)"}"#;
        let q = PayloadStatus {
            latest_valid_hash: None,
            status: PayloadStatusEnum::Invalid {
                validation_error: PayloadValidationError::InvalidStateRoot {
                    remote: "0x3f77fb29ce67436532fee970e1add8f5cc80e8878c79b967af53b1fd92a0cab7"
                        .parse()
                        .unwrap(),
                    local: "0x603b9628dabdaadb442a3bb3d7e0360efc110e1948472909230909f1690fed17"
                        .parse()
                        .unwrap(),
                }
                .to_string(),
            },
        };
        assert_eq!(q, serde_json::from_str(s).unwrap());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_roundtrip_legacy_txs_payload_v1() {
        // pulled from hive tests
        let s = r#"{"parentHash":"0x67ead97eb79b47a1638659942384143f36ed44275d4182799875ab5a87324055","feeRecipient":"0x0000000000000000000000000000000000000000","stateRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","receiptsRoot":"0x4e3c608a9f2e129fccb91a1dae7472e78013b8e654bccc8d224ce3d63ae17006","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","prevRandao":"0x44bb4b98c59dbb726f96ffceb5ee028dcbe35b9bba4f9ffd56aeebf8d1e4db62","blockNumber":"0x1","gasLimit":"0x2fefd8","gasUsed":"0xa860","timestamp":"0x1235","extraData":"0x8b726574682f76302e312e30","baseFeePerGas":"0x342770c0","blockHash":"0x5655011482546f16b2312ef18e9fad03d6a52b1be95401aea884b222477f9e64","transactions":["0xf865808506fc23ac00830124f8940000000000000000000000000000000000000316018032a044b25a8b9b247d01586b3d59c71728ff49c9b84928d9e7fa3377ead3b5570b5da03ceac696601ff7ee6f5fe8864e2998db9babdf5eeba1a0cd5b4d44b3fcbd181b"]}"#;
        let payload: ExecutionPayloadV1 = serde_json::from_str(s).unwrap();
        assert_eq!(serde_json::to_string(&payload).unwrap(), s);

        let any_payload: ExecutionPayload = serde_json::from_str(s).unwrap();
        assert_eq!(any_payload, payload.into());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_roundtrip_legacy_txs_payload_v3() {
        // pulled from hive tests - modified with 4844 fields
        let s = r#"{"parentHash":"0x67ead97eb79b47a1638659942384143f36ed44275d4182799875ab5a87324055","feeRecipient":"0x0000000000000000000000000000000000000000","stateRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","receiptsRoot":"0x4e3c608a9f2e129fccb91a1dae7472e78013b8e654bccc8d224ce3d63ae17006","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","prevRandao":"0x44bb4b98c59dbb726f96ffceb5ee028dcbe35b9bba4f9ffd56aeebf8d1e4db62","blockNumber":"0x1","gasLimit":"0x2fefd8","gasUsed":"0xa860","timestamp":"0x1235","extraData":"0x8b726574682f76302e312e30","baseFeePerGas":"0x342770c0","blockHash":"0x5655011482546f16b2312ef18e9fad03d6a52b1be95401aea884b222477f9e64","transactions":["0xf865808506fc23ac00830124f8940000000000000000000000000000000000000316018032a044b25a8b9b247d01586b3d59c71728ff49c9b84928d9e7fa3377ead3b5570b5da03ceac696601ff7ee6f5fe8864e2998db9babdf5eeba1a0cd5b4d44b3fcbd181b"],"withdrawals":[],"blobGasUsed":"0xb10b","excessBlobGas":"0xb10b"}"#;
        let payload: ExecutionPayloadV3 = serde_json::from_str(s).unwrap();
        assert_eq!(serde_json::to_string(&payload).unwrap(), s);

        let any_payload: ExecutionPayload = serde_json::from_str(s).unwrap();
        assert_eq!(any_payload, payload.into());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_roundtrip_enveloped_txs_payload_v1() {
        // pulled from hive tests
        let s = r#"{"parentHash":"0x67ead97eb79b47a1638659942384143f36ed44275d4182799875ab5a87324055","feeRecipient":"0x0000000000000000000000000000000000000000","stateRoot":"0x76a03cbcb7adce07fd284c61e4fa31e5e786175cefac54a29e46ec8efa28ea41","receiptsRoot":"0x4e3c608a9f2e129fccb91a1dae7472e78013b8e654bccc8d224ce3d63ae17006","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","prevRandao":"0x028111cb7d25918386a69656b3d17b2febe95fd0f11572c1a55c14f99fdfe3df","blockNumber":"0x1","gasLimit":"0x2fefd8","gasUsed":"0xa860","timestamp":"0x1235","extraData":"0x8b726574682f76302e312e30","baseFeePerGas":"0x342770c0","blockHash":"0xa6f40ed042e61e88e76125dede8fff8026751ea14454b68fb534cea99f2b2a77","transactions":["0xf865808506fc23ac00830124f8940000000000000000000000000000000000000316018032a044b25a8b9b247d01586b3d59c71728ff49c9b84928d9e7fa3377ead3b5570b5da03ceac696601ff7ee6f5fe8864e2998db9babdf5eeba1a0cd5b4d44b3fcbd181b"]}"#;
        let payload: ExecutionPayloadV1 = serde_json::from_str(s).unwrap();
        assert_eq!(serde_json::to_string(&payload).unwrap(), s);

        let any_payload: ExecutionPayload = serde_json::from_str(s).unwrap();
        assert_eq!(any_payload, payload.into());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_roundtrip_enveloped_txs_payload_v3() {
        // pulled from hive tests - modified with 4844 fields
        let s = r#"{"parentHash":"0x67ead97eb79b47a1638659942384143f36ed44275d4182799875ab5a87324055","feeRecipient":"0x0000000000000000000000000000000000000000","stateRoot":"0x76a03cbcb7adce07fd284c61e4fa31e5e786175cefac54a29e46ec8efa28ea41","receiptsRoot":"0x4e3c608a9f2e129fccb91a1dae7472e78013b8e654bccc8d224ce3d63ae17006","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","prevRandao":"0x028111cb7d25918386a69656b3d17b2febe95fd0f11572c1a55c14f99fdfe3df","blockNumber":"0x1","gasLimit":"0x2fefd8","gasUsed":"0xa860","timestamp":"0x1235","extraData":"0x8b726574682f76302e312e30","baseFeePerGas":"0x342770c0","blockHash":"0xa6f40ed042e61e88e76125dede8fff8026751ea14454b68fb534cea99f2b2a77","transactions":["0xf865808506fc23ac00830124f8940000000000000000000000000000000000000316018032a044b25a8b9b247d01586b3d59c71728ff49c9b84928d9e7fa3377ead3b5570b5da03ceac696601ff7ee6f5fe8864e2998db9babdf5eeba1a0cd5b4d44b3fcbd181b"],"withdrawals":[],"blobGasUsed":"0xb10b","excessBlobGas":"0xb10b"}"#;
        let payload: ExecutionPayloadV3 = serde_json::from_str(s).unwrap();
        assert_eq!(serde_json::to_string(&payload).unwrap(), s);

        let any_payload: ExecutionPayload = serde_json::from_str(s).unwrap();
        assert_eq!(any_payload, payload.into());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_roundtrip_execution_payload_envelope_v3() {
        // pulled from a geth response getPayloadV3 in hive tests
        let response = r#"{"executionPayload":{"parentHash":"0xe927a1448525fb5d32cb50ee1408461a945ba6c39bd5cf5621407d500ecc8de9","feeRecipient":"0x0000000000000000000000000000000000000000","stateRoot":"0x10f8a0830000e8edef6d00cc727ff833f064b1950afd591ae41357f97e543119","receiptsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","prevRandao":"0xe0d8b4521a7da1582a713244ffb6a86aa1726932087386e2dc7973f43fc6cb24","blockNumber":"0x1","gasLimit":"0x2ffbd2","gasUsed":"0x0","timestamp":"0x1235","extraData":"0xd883010d00846765746888676f312e32312e30856c696e7578","baseFeePerGas":"0x342770c0","blockHash":"0x44d0fa5f2f73a938ebb96a2a21679eb8dea3e7b7dd8fd9f35aa756dda8bf0a8a","transactions":[],"withdrawals":[],"blobGasUsed":"0x0","excessBlobGas":"0x0"},"blockValue":"0x0","blobsBundle":{"commitments":[],"proofs":[],"blobs":[]},"shouldOverrideBuilder":false}"#;
        let envelope: ExecutionPayloadEnvelopeV3 = serde_json::from_str(response).unwrap();
        assert_eq!(serde_json::to_string(&envelope).unwrap(), response);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_deserialize_execution_payload_input_v2() {
        let response = r#"
{
  "baseFeePerGas": "0x173b30b3",
  "blockHash": "0x99d486755fd046ad0bbb60457bac93d4856aa42fa00629cc7e4a28b65b5f8164",
  "blockNumber": "0xb",
  "extraData": "0xd883010d01846765746888676f312e32302e33856c696e7578",
  "feeRecipient": "0x0000000000000000000000000000000000000000",
  "gasLimit": "0x405829",
  "gasUsed": "0x3f0ca0",
  "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
  "parentHash": "0xfe34aaa2b869c66a727783ee5ad3e3983b6ef22baf24a1e502add94e7bcac67a",
  "prevRandao": "0x74132c32fe3ab9a470a8352544514d21b6969e7749f97742b53c18a1b22b396c",
  "receiptsRoot": "0x6a5c41dc55a1bd3e74e7f6accc799efb08b00c36c15265058433fcea6323e95f",
  "stateRoot": "0xde3b357f5f099e4c33d0343c9e9d204d663d7bd9c65020a38e5d0b2a9ace78a2",
  "timestamp": "0x6507d6b4",
  "transactions": [
    "0xf86d0a8458b20efd825208946177843db3138ae69679a54b95cf345ed759450d8806f3e8d87878800080820a95a0f8bddb1dcc4558b532ff747760a6f547dd275afdbe7bdecc90680e71de105757a014f34ba38c180913c0543b0ac2eccfb77cc3f801a535008dc50e533fbe435f53",
    "0xf86d0b8458b20efd82520894687704db07e902e9a8b3754031d168d46e3d586e8806f3e8d87878800080820a95a0e3108f710902be662d5c978af16109961ffaf2ac4f88522407d40949a9574276a0205719ed21889b42ab5c1026d40b759a507c12d92db0d100fa69e1ac79137caa",
    "0xf86d0c8458b20efd8252089415e6a5a2e131dd5467fa1ff3acd104f45ee5940b8806f3e8d87878800080820a96a0af556ba9cda1d686239e08c24e169dece7afa7b85e0948eaa8d457c0561277fca029da03d3af0978322e54ac7e8e654da23934e0dd839804cb0430f8aaafd732dc",
    "0xf8521784565adcb7830186a0808080820a96a0ec782872a673a9fe4eff028a5bdb30d6b8b7711f58a187bf55d3aec9757cb18ea001796d373da76f2b0aeda72183cce0ad070a4f03aa3e6fee4c757a9444245206",
    "0xf8521284565adcb7830186a0808080820a95a08a0ea89028eff02596b385a10e0bd6ae098f3b281be2c95a9feb1685065d7384a06239d48a72e4be767bd12f317dd54202f5623a33e71e25a87cb25dd781aa2fc8",
    "0xf8521384565adcb7830186a0808080820a95a0784dbd311a82f822184a46f1677a428cbe3a2b88a798fb8ad1370cdbc06429e8a07a7f6a0efd428e3d822d1de9a050b8a883938b632185c254944dd3e40180eb79"
  ],
  "withdrawals": []
}
        "#;
        let payload: ExecutionPayloadInputV2 = serde_json::from_str(response).unwrap();
        assert_eq!(payload.withdrawals, Some(vec![]));

        let response = r#"
{
  "baseFeePerGas": "0x173b30b3",
  "blockHash": "0x99d486755fd046ad0bbb60457bac93d4856aa42fa00629cc7e4a28b65b5f8164",
  "blockNumber": "0xb",
  "extraData": "0xd883010d01846765746888676f312e32302e33856c696e7578",
  "feeRecipient": "0x0000000000000000000000000000000000000000",
  "gasLimit": "0x405829",
  "gasUsed": "0x3f0ca0",
  "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
  "parentHash": "0xfe34aaa2b869c66a727783ee5ad3e3983b6ef22baf24a1e502add94e7bcac67a",
  "prevRandao": "0x74132c32fe3ab9a470a8352544514d21b6969e7749f97742b53c18a1b22b396c",
  "receiptsRoot": "0x6a5c41dc55a1bd3e74e7f6accc799efb08b00c36c15265058433fcea6323e95f",
  "stateRoot": "0xde3b357f5f099e4c33d0343c9e9d204d663d7bd9c65020a38e5d0b2a9ace78a2",
  "timestamp": "0x6507d6b4",
  "transactions": [
    "0xf86d0a8458b20efd825208946177843db3138ae69679a54b95cf345ed759450d8806f3e8d87878800080820a95a0f8bddb1dcc4558b532ff747760a6f547dd275afdbe7bdecc90680e71de105757a014f34ba38c180913c0543b0ac2eccfb77cc3f801a535008dc50e533fbe435f53",
    "0xf86d0b8458b20efd82520894687704db07e902e9a8b3754031d168d46e3d586e8806f3e8d87878800080820a95a0e3108f710902be662d5c978af16109961ffaf2ac4f88522407d40949a9574276a0205719ed21889b42ab5c1026d40b759a507c12d92db0d100fa69e1ac79137caa",
    "0xf86d0c8458b20efd8252089415e6a5a2e131dd5467fa1ff3acd104f45ee5940b8806f3e8d87878800080820a96a0af556ba9cda1d686239e08c24e169dece7afa7b85e0948eaa8d457c0561277fca029da03d3af0978322e54ac7e8e654da23934e0dd839804cb0430f8aaafd732dc",
    "0xf8521784565adcb7830186a0808080820a96a0ec782872a673a9fe4eff028a5bdb30d6b8b7711f58a187bf55d3aec9757cb18ea001796d373da76f2b0aeda72183cce0ad070a4f03aa3e6fee4c757a9444245206",
    "0xf8521284565adcb7830186a0808080820a95a08a0ea89028eff02596b385a10e0bd6ae098f3b281be2c95a9feb1685065d7384a06239d48a72e4be767bd12f317dd54202f5623a33e71e25a87cb25dd781aa2fc8",
    "0xf8521384565adcb7830186a0808080820a95a0784dbd311a82f822184a46f1677a428cbe3a2b88a798fb8ad1370cdbc06429e8a07a7f6a0efd428e3d822d1de9a050b8a883938b632185c254944dd3e40180eb79"
  ]
}
        "#;
        let payload: ExecutionPayloadInputV2 = serde_json::from_str(response).unwrap();
        assert_eq!(payload.withdrawals, None);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_deserialize_v3_with_unknown_fields() {
        let input = r#"
{
    "parentHash": "0xaaa4c5b574f37e1537c78931d1bca24a4d17d4f29f1ee97e1cd48b704909de1f",
    "feeRecipient": "0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba",
    "stateRoot": "0x308ee9c5c6fab5e3d08763a3b5fe0be8ada891fa5010a49a3390e018dd436810",
    "receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
    "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "prevRandao": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "blockNumber": "0xf",
    "gasLimit": "0x16345785d8a0000",
    "gasUsed": "0x0",
    "timestamp": "0x3a97",
    "extraData": "0x",
    "baseFeePerGas": "0x7",
    "blockHash": "0x38bb6ba645c7e6bd970f9c7d492fafe1e04d85349054cb48d16c9d2c3e3cd0bf",
    "transactions": [],
    "withdrawals": [],
    "excessBlobGas": "0x0",
    "blobGasUsed": "0x0"
}
        "#;

        // ensure that deserializing this succeeds
        let _payload_res: ExecutionPayloadV3 = serde_json::from_str(input).unwrap();

        // construct a payload with a random field in the middle
        let input = r#"
{
    "parentHash": "0xaaa4c5b574f37e1537c78931d1bca24a4d17d4f29f1ee97e1cd48b704909de1f",
    "feeRecipient": "0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba",
    "stateRoot": "0x308ee9c5c6fab5e3d08763a3b5fe0be8ada891fa5010a49a3390e018dd436810",
    "receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
    "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "prevRandao": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "blockNumber": "0xf",
    "gasLimit": "0x16345785d8a0000",
    "gasUsed": "0x0",
    "timestamp": "0x3a97",
    "extraData": "0x",
    "baseFeePerGas": "0x7",
    "blockHash": "0x38bb6ba645c7e6bd970f9c7d492fafe1e04d85349054cb48d16c9d2c3e3cd0bf",
    "transactions": [],
    "withdrawals": [],
    "randomStuff": [],
    "excessBlobGas": "0x0",
    "blobGasUsed": "0x0"
}
        "#;

        // ensure that deserializing this fails
        let _payload_res = serde_json::from_str::<ExecutionPayloadV3>(input).unwrap_err();

        // construct a payload with a random field at the end
        let input = r#"
{
    "parentHash": "0xaaa4c5b574f37e1537c78931d1bca24a4d17d4f29f1ee97e1cd48b704909de1f",
    "feeRecipient": "0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba",
    "stateRoot": "0x308ee9c5c6fab5e3d08763a3b5fe0be8ada891fa5010a49a3390e018dd436810",
    "receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
    "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "prevRandao": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "blockNumber": "0xf",
    "gasLimit": "0x16345785d8a0000",
    "gasUsed": "0x0",
    "timestamp": "0x3a97",
    "extraData": "0x",
    "baseFeePerGas": "0x7",
    "blockHash": "0x38bb6ba645c7e6bd970f9c7d492fafe1e04d85349054cb48d16c9d2c3e3cd0bf",
    "transactions": [],
    "withdrawals": [],
    "randomStuff": [],
    "excessBlobGas": "0x0",
    "blobGasUsed": "0x0"
    "moreRandomStuff": "0x0",
}
        "#;

        // ensure that deserializing this fails
        let _payload_res = serde_json::from_str::<ExecutionPayloadV3>(input).unwrap_err();
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_deserialize_v2_input_with_blob_fields() {
        let input = r#"
{
    "parentHash": "0xaaa4c5b574f37e1537c78931d1bca24a4d17d4f29f1ee97e1cd48b704909de1f",
    "feeRecipient": "0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba",
    "stateRoot": "0x308ee9c5c6fab5e3d08763a3b5fe0be8ada891fa5010a49a3390e018dd436810",
    "receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
    "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "prevRandao": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "blockNumber": "0xf",
    "gasLimit": "0x16345785d8a0000",
    "gasUsed": "0x0",
    "timestamp": "0x3a97",
    "extraData": "0x",
    "baseFeePerGas": "0x7",
    "blockHash": "0x38bb6ba645c7e6bd970f9c7d492fafe1e04d85349054cb48d16c9d2c3e3cd0bf",
    "transactions": [],
    "withdrawals": [],
    "excessBlobGas": "0x0",
    "blobGasUsed": "0x0"
}
        "#;

        // ensure that deserializing this (it includes blob fields) fails
        let payload_res: Result<ExecutionPayloadInputV2, serde_json::Error> =
            serde_json::from_str(input);
        assert!(payload_res.is_err());
    }

    // <https://github.com/paradigmxyz/reth/issues/6036>
    #[test]
    #[cfg(feature = "serde")]
    fn deserialize_op_base_payload() {
        let payload = r#"{"parentHash":"0x24e8df372a61cdcdb1a163b52aaa1785e0c869d28c3b742ac09e826bbb524723","feeRecipient":"0x4200000000000000000000000000000000000011","stateRoot":"0x9a5db45897f1ff1e620a6c14b0a6f1b3bcdbed59f2adc516a34c9a9d6baafa71","receiptsRoot":"0x8af6f74835d47835deb5628ca941d00e0c9fd75585f26dabdcb280ec7122e6af","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","prevRandao":"0xf37b24eeff594848072a05f74c8600001706c83e489a9132e55bf43a236e42ec","blockNumber":"0xe3d5d8","gasLimit":"0x17d7840","gasUsed":"0xb705","timestamp":"0x65a118c0","extraData":"0x","baseFeePerGas":"0x7a0ff32","blockHash":"0xf5c147b2d60a519b72434f0a8e082e18599021294dd9085d7597b0ffa638f1c0","withdrawals":[],"transactions":["0x7ef90159a05ba0034ffdcb246703298224564720b66964a6a69d0d7e9ffd970c546f7c048094deaddeaddeaddeaddeaddeaddeaddeaddead00019442000000000000000000000000000000000000158080830f424080b90104015d8eb900000000000000000000000000000000000000000000000000000000009e1c4a0000000000000000000000000000000000000000000000000000000065a11748000000000000000000000000000000000000000000000000000000000000000a4b479e5fa8d52dd20a8a66e468b56e993bdbffcccf729223aabff06299ab36db000000000000000000000000000000000000000000000000000000000000000400000000000000000000000073b4168cc87f35cc239200a20eb841cded23493b000000000000000000000000000000000000000000000000000000000000083400000000000000000000000000000000000000000000000000000000000f4240"]}"#;
        let _payload = serde_json::from_str::<ExecutionPayloadInputV2>(payload).unwrap();
    }
}
