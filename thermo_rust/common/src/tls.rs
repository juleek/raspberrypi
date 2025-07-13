use anyhow::{anyhow, Context, Result};


//
// ===========================================================================================================
// Helpers

type Validity = std::ops::Range<chrono::NaiveDate>;

fn set_validity(params: &mut rcgen::CertificateParams, validity: Validity) {
   use chrono::Datelike;
   params.not_after = rcgen::date_time_ymd(
      validity.end.year(),
      validity.end.month().try_into().unwrap(),
      validity.end.day().try_into().unwrap(),
   );
   params.not_before = rcgen::date_time_ymd(
      validity.start.year(),
      validity.start.month().try_into().unwrap(),
      validity.start.day().try_into().unwrap(),
   );
}

fn validity_from_days(days: i64) -> Validity {
   let start = chrono::Utc::now();
   let end = start + chrono::TimeDelta::days(days);
   start.date_naive()..end.date_naive()
}

fn load_ca_cert_and_key(
   ca_cert: &std::path::Path,
   ca_key: &std::path::Path,
) -> Result<(rcgen::Certificate, rcgen::KeyPair)> {
   let ca_key = rcgen::KeyPair::from_pem(
      &std::fs::read_to_string(ca_key).with_context(|| anyhow!("Failed to read from {ca_key:?}"))?,
   )
   .with_context(|| anyhow!("Failed to deserialise ca key from pem: {ca_key:?}"))?;

   let params = rcgen::CertificateParams::from_ca_cert_pem(
      &std::fs::read_to_string(ca_cert).with_context(|| anyhow!("Failed to read from {ca_cert:?}"))?,
   )
   .with_context(|| anyhow!("Failed to deserialise ca certificate from pem: {ca_cert:?}"))?;

   let ca_cert = params
      .self_signed(&ca_key)
      .with_context(|| anyhow!("Failed to genrate self-signed CA cert"))?;

   Ok((ca_cert, ca_key))
}

fn save_in_file(path: &std::path::Path, content: &str) -> Result<()> {
   use std::io::Write;

   if let Some(parent) = path.parent() && parent.as_os_str().is_empty() == false {
         std::fs::create_dir_all(parent)
            .with_context(|| anyhow!("Failed to create directories: {:?}", parent))?;
   }

   let mut file = std::fs::File::create(path).with_context(|| anyhow!("Failed to create file: {path:?}"))?;
   file.write_all(content.as_bytes())?;
   Ok(())
}

fn read_file(path: &std::path::Path) -> Result<Vec<u8>> {
   use std::io::Read;

   let mut file = std::fs::File::open(path).with_context(|| anyhow!("Failed to open: {path:?}"))?;
   let mut contents = Vec::new();
   file.read_to_end(&mut contents).with_context(|| anyhow!("Failed to read from {path:?}"))?;
   Ok(contents)
}

//
// ===========================================================================================================
// Main functions


pub fn generate_ca(subject: &str, validity: Validity) -> Result<(rcgen::Certificate, rcgen::KeyPair)> {
   let mut params = rcgen::CertificateParams::default();
   set_validity(&mut params, validity);

   params.distinguished_name = {
      let mut dn = rcgen::DistinguishedName::new();
      dn.push(rcgen::DnType::CommonName, subject);
      dn
   };

   params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

   params.key_usages = vec![rcgen::KeyUsagePurpose::KeyCertSign, rcgen::KeyUsagePurpose::CrlSign];


   let key_pair = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P384_SHA384)
      .with_context(|| anyhow!("Failed to generate key_pair"))?;
   let cert = params
      .self_signed(&key_pair)
      .with_context(|| anyhow!("Failed to generate self signed ca cert"))?;
   Ok((cert, key_pair))
}

pub fn generate_server(
   subject: &str,
   validity: Validity,
   san_ips: &[std::net::IpAddr],
   san_hosts: &[impl AsRef<str>],
   ca_cert: &rcgen::Certificate,
   ca_key: &rcgen::KeyPair,
) -> Result<(rcgen::Certificate, rcgen::KeyPair)> {
   let mut params = rcgen::CertificateParams::default();
   set_validity(&mut params, validity);

   params.distinguished_name = {
      let mut dn = rcgen::DistinguishedName::new();
      dn.push(rcgen::DnType::CommonName, subject);
      dn
   };

   params.subject_alt_names = std::iter::empty()
      .chain(san_ips.iter().map(|i| rcgen::SanType::IpAddress(*i)))
      .chain(san_hosts.iter().map(|h| rcgen::SanType::DnsName(h.as_ref().try_into().unwrap())))
      .collect();

   params.is_ca = rcgen::IsCa::ExplicitNoCa;

   params.key_usages = vec![
      rcgen::KeyUsagePurpose::DigitalSignature,
      rcgen::KeyUsagePurpose::ContentCommitment,
      rcgen::KeyUsagePurpose::KeyEncipherment,
      rcgen::KeyUsagePurpose::KeyAgreement,
   ];

   params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];

   let key_pair = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P384_SHA384)
      .with_context(|| anyhow!("Failed to generate key_pair"))?;
   let cert = params
      .signed_by(&key_pair, ca_cert, ca_key)
      .with_context(|| anyhow!("Failed to generate and signed server cert"))?;

   Ok((cert, key_pair))
}

pub fn generate_client(
   subject: &str,
   validity: Validity,
   ca_cert: &rcgen::Certificate,
   ca_key: &rcgen::KeyPair,
) -> Result<(rcgen::Certificate, rcgen::KeyPair)> {
   let mut params = rcgen::CertificateParams::default();
   set_validity(&mut params, validity);
   // not_before and not_after
   params.distinguished_name = {
      let mut dn = rcgen::DistinguishedName::new();
      dn.push(rcgen::DnType::CommonName, subject);
      dn
   };

   params.is_ca = rcgen::IsCa::ExplicitNoCa;

   params.key_usages = vec![
      rcgen::KeyUsagePurpose::DigitalSignature,
      rcgen::KeyUsagePurpose::ContentCommitment,
      rcgen::KeyUsagePurpose::KeyEncipherment,
      rcgen::KeyUsagePurpose::KeyAgreement,
   ];

   params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];

   let key_pair = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P384_SHA384)
      .with_context(|| anyhow!("Failed to generate key_pair"))?;
   let cert = params
      .signed_by(&key_pair, ca_cert, ca_key)
      .with_context(|| anyhow!("Failed to generate and signed client cert"))?;

   Ok((cert, key_pair))
}


pub fn generate_subject(prefix: &str, len: usize, subject: &Option<String>) -> String {
   let Some(subject) = subject else {
      return crate::generate_random_string(prefix, len);
   };
   subject.clone()
}


//
// ===========================================================================================================
// Ca helper struct

pub struct Ca {
   cert: rcgen::Certificate,
   key_pair: rcgen::KeyPair,
   validity: Validity,
}

impl Ca {
   pub fn new(days_valid: i64) -> Result<Self> {
      let validity = validity_from_days(days_valid);
      let (cert, key_pair) = generate_ca(&crate::generate_random_string("CA-", 3), validity.clone())?;

      Ok(Self {
         cert,
         key_pair,
         validity,
      })
   }

   pub fn server(
      &self,
      san_ips: &[std::net::IpAddr],
      san_hosts: &[impl AsRef<str>],
   ) -> Result<(rcgen::Certificate, rcgen::KeyPair)> {
      generate_server(
         &crate::generate_random_string("SRV-", 5),
         self.validity.clone(),
         san_ips,
         san_hosts,
         &self.cert,
         &self.key_pair,
      )
   }

   pub fn client(&self) -> Result<(rcgen::Certificate, rcgen::KeyPair)> {
      generate_client(
         &crate::generate_random_string("CLI-", 7),
         self.validity.clone(),
         &self.cert,
         &self.key_pair,
      )
   }
}


//
// ===========================================================================================================
// CLI options for generating certificates

/// Generate CA key and cert files.
#[derive(Debug, clap::Parser)]
pub struct GenCaOpts {
   /// Output Cert path
   #[arg(long)]
   ca_cert: std::path::PathBuf,

   /// Output Key path
   #[arg(long)]
   ca_key: std::path::PathBuf,

   /// Subject, if not specified, genrated randomly
   #[arg(long)]
   subject: Option<String>,

   /// Valid for this number of days.
   #[arg(long, default_value_t = 365 * 20)]
   valid: i64,
}


impl GenCaOpts {
   pub async fn run(&self) -> Result<()> {
      let (cert, key_pair) =
         generate_ca(&generate_subject("CA-", 3, &self.subject), validity_from_days(self.valid))?;

      save_in_file(&self.ca_cert, &cert.pem())?;
      save_in_file(&self.ca_key, &key_pair.serialize_pem())?;
      Ok(())
   }
}




/// Generate Server key and cert files.
#[derive(Debug, clap::Parser)]
pub struct GenServerOpts {
   /// Input: CA Cert path
   #[arg(long)]
   ca_cert: std::path::PathBuf,

   /// Input: CA Key path
   #[arg(long)]
   ca_key: std::path::PathBuf,

   /// Output: CA Cert path
   #[arg(long)]
   cert: std::path::PathBuf,

   /// Output: CA Key path
   #[arg(long)]
   key: std::path::PathBuf,

   /// Subject, if not specified, genrated randomly
   #[arg(long)]
   subject: Option<String>,

   /// Valid for this number of days.
   #[arg(long, default_value_t = 365 * 20)]
   valid: i64,

   /// Provide hosts / dns names here
   #[arg(long)]
   san_hosts: Vec<String>,

   /// Provide IPs here
   #[arg(long)]
   san_ips: Vec<String>,
}


impl GenServerOpts {
   pub fn san_ips(&self) -> Result<Vec<std::net::IpAddr>, std::net::AddrParseError> {
      self.san_ips.iter().map(|ip| ip.parse()).collect()
   }

   pub async fn run(&self) -> Result<()> {
      let (ca_cert, ca_key) = load_ca_cert_and_key(&self.ca_cert, &self.ca_key)?;
      let (cert, key_pair) = generate_server(
         &generate_subject("SRV-", 5, &self.subject),
         validity_from_days(self.valid),
         &self.san_ips()?,
         &self.san_hosts,
         &ca_cert,
         &ca_key,
      )?;

      save_in_file(&self.cert, &cert.pem())?;
      save_in_file(&self.key, &key_pair.serialize_pem())?;
      Ok(())
   }
}




/// Generate Client key and cert files.
#[derive(Debug, clap::Parser)]
pub struct GenClientOpts {
   /// Input: CA Cert path
   #[arg(long)]
   ca_cert: std::path::PathBuf,

   /// Input: CA Key path
   #[arg(long)]
   ca_key: std::path::PathBuf,

   /// Output: CA Cert path
   #[arg(long)]
   cert: std::path::PathBuf,

   /// Output: CA Key path
   #[arg(long)]
   key: std::path::PathBuf,

   /// Subject, if not specified, genrated randomly
   #[arg(long)]
   subject: Option<String>,

   /// Valid for this number of days.
   #[arg(long, default_value_t = 365 * 20)]
   valid: i64,
}


impl GenClientOpts {
   pub async fn run(&self) -> Result<()> {
      let (ca_cert, ca_key) = load_ca_cert_and_key(&self.ca_cert, &self.ca_key)?;
      let (cert, key_pair) = generate_client(
         &generate_subject("SRV-", 5, &self.subject),
         validity_from_days(self.valid),
         &ca_cert,
         &ca_key,
      )?;

      save_in_file(&self.cert, &cert.pem())?;
      save_in_file(&self.key, &key_pair.serialize_pem())?;
      Ok(())
   }
}




//
// ===========================================================================================================
// CLI options for accepting certificate & key paths


#[derive(clap::Parser, Debug, Clone)]
pub struct ServerArgs {
   /// Path to PEM-encoded CA certificate
   #[clap(long)]
   tls_ca_cert: std::path::PathBuf,

   /// Path to PEM-encoded server certificate
   #[clap(long)]
   tls_server_cert: std::path::PathBuf,

   /// Path to PEM-encoded server key
   #[clap(long)]
   tls_server_key: std::path::PathBuf,
}

impl ServerArgs {
   pub fn server_tls_config(&self) -> Result<tonic::transport::ServerTlsConfig> {
      let ca = read_file(&self.tls_ca_cert)?;
      let cert = read_file(&self.tls_server_cert)?;
      let key = read_file(&self.tls_server_key)?;

      let identity = tonic::transport::Identity::from_pem(cert, key);
      let ca = tonic::transport::Certificate::from_pem(ca);

      Ok(tonic::transport::ServerTlsConfig::new().identity(identity).client_ca_root(ca))
   }
}



pub struct ClientConfigProvider {
   identity: tonic::transport::Identity,
   ca: tonic::transport::Certificate,
}
impl ClientConfigProvider {
   pub fn new(identity: tonic::transport::Identity, ca: tonic::transport::Certificate) -> Self {
      Self { identity, ca }
   }

   pub fn create_for(&self, server_ip_or_host: &str) -> tonic::transport::ClientTlsConfig {
      tonic::transport::ClientTlsConfig::new()
         .domain_name(server_ip_or_host)
         .identity(self.identity.clone())
         .ca_certificate(self.ca.clone())
   }
}

#[derive(clap::Parser, Debug, Clone)]
pub struct ClientArgs {
   /// Path to PEM-encoded CA certificate
   #[clap(long)]
   tls_ca_cert: std::path::PathBuf,

   /// Path to PEM-encoded client certificate
   #[clap(long)]
   tls_client_cert: std::path::PathBuf,

   /// Path to PEM-encoded client key
   #[clap(long)]
   tls_client_key: std::path::PathBuf,
}

impl ClientArgs {
   pub fn client_config_provider(&self) -> Result<ClientConfigProvider> {
      let ca = read_file(&self.tls_ca_cert)?;
      let cert = read_file(&self.tls_client_cert)?;
      let key = read_file(&self.tls_client_key)?;

      let identity = tonic::transport::Identity::from_pem(cert, key);
      let ca = tonic::transport::Certificate::from_pem(ca);

      Ok(ClientConfigProvider::new(identity, ca))
   }
}
