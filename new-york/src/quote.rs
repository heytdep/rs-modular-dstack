use base64::{prelude::BASE64_STANDARD, Engine};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QuoteVerificationResult {
    pub header: Header,
    pub td_quote_body: TdQuoteBody,
    pub signed_data_size: u32,
    pub signed_data: SignedData,
    pub extra_bytes: String,
}

#[derive(Debug, Deserialize)]
pub struct Header {
    pub version: u8,
    pub attestation_key_type: u8,
    pub tee_type: u8,
    pub qe_svn: String,
    pub pce_svn: String,
    pub qe_vendor_id: String,
    pub user_data: String,
}

#[derive(Debug, Deserialize)]
pub struct TdQuoteBody {
    pub tee_tcb_svn: String,
    pub mr_seam: String,
    pub mr_signer_seam: String,
    pub seam_attributes: String,
    pub td_attributes: String,
    pub xfam: String,
    pub mr_td: String,
    pub mr_config_id: String,
    pub mr_owner: String,
    pub mr_owner_config: String,
    pub rtmrs: Vec<String>,
    pub report_data: String,
}

#[derive(Debug, Deserialize)]
pub struct SignedData {
    pub signature: String,
    pub ecdsa_attestation_key: String,
    pub certification_data: CertificationData,
}

#[derive(Debug, Deserialize)]
pub struct CertificationData {
    pub certificate_data_type: u8,
    pub size: u32,
    pub qe_report_certification_data: QeReportCertificationData,
}

#[derive(Debug, Deserialize)]
pub struct QeReportCertificationData {
    pub qe_report: QeReport,
    pub qe_report_signature: String,
    pub qe_auth_data: QeAuthData,
    pub pck_certificate_chain_data: PckCertificateChainData,
}

#[derive(Debug, Deserialize)]
pub struct QeReport {
    pub cpu_svn: String,
    pub reserved1: String,
    pub attributes: String,
    pub mr_enclave: String,
    pub reserved2: String,
    pub mr_signer: String,
    pub reserved3: String,
    pub isv_prod_id: u16,
    pub isv_svn: u16,
    pub reserved4: String,
    pub report_data: String,
}

#[derive(Debug, Deserialize)]
pub struct QeAuthData {
    pub parsed_data_size: u32,
    pub data: String,
}

#[derive(Debug, Deserialize)]
pub struct PckCertificateChainData {
    pub certificate_data_type: u8,
    pub size: u32,
    pub pck_cert_chain: String,
}

impl QuoteVerificationResult {
    pub fn get_appdata(&self) -> [u8; 32] {
        BASE64_STANDARD
            .decode(&self.td_quote_body.report_data)
            .unwrap()[0..32]
            .try_into()
            .unwrap()
    }
}
