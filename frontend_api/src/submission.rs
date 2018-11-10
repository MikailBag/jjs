use crate::base::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeclareRequest {
    pub check_sum: CheckSum,
    pub size: u64,
    pub toolchain: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeclareSuccess {
    pub upload_token: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DeclareFail {
    ContestOver,
    Denied,
    UnknownToolchain,
    SizeLimitExceeded,
    UnknownDigest,
    Other,
}

pub type DeclareResult = Result<DeclareSuccess, DeclareFail>;

#[derive(Debug, Serialize, Deserialize)]
pub struct PutChunkRequest {
    pub upload_token: u64,
    pub chunk: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PutChunkSuccess {}

#[derive(Debug, Serialize, Deserialize)]
pub enum PutChunkFail {
    IncorrectSize,
    UnknownUploadToken,
    Timeout,
}

pub type PutChunkResult = Result<PutChunkSuccess, PutChunkFail>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishRequest {
    pub upload_token: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishSuccess {
    pub submission_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FinishFail {
    UnknownUploadToken,
    UploadNotFinished,
    Timeout,
    IncorrectDigest,
}

pub type FinishResult = Result<FinishSuccess, FinishFail>;

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadRequest {
    ///base64
    pub data: String,
    pub toolchain: String,
}

/*
#[derive(Debug, Serialize, Deserialize)]
pub enum UploadFail {
    ContestOver,
    Denied,
    UnknownToolchain,
    SizeLimitExceeded,
    Other,
}*/


#[derive(Debug, Serialize, Deserialize)]
pub enum SubmissionRequest {
    Declare(DeclareRequest),
    PutChunk(PutChunkRequest),
    Finish(FinishRequest),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SubmissionResult {
    Declare(DeclareResult),
    PutChunk(PutChunkResult),
    Finish(FinishResult),
}