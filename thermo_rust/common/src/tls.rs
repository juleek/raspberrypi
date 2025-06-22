use std::vec;

use anyhow::{anyhow, Context, Result};

pub fn generate_ca(
   subject: &str,
   validity: std::ops::Range<chrono::NaiveDate>,
) -> Result<(rcgen::Certificate, rcgen::KeyPair)> {
   let mut params = rcgen::CertificateParams::default();
   set_validity(&mut params, validity);
   // not_before and not_after
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
   Ok((key_pair, cert))
}

pub fn generate_server(
   subject: &str,
   validity: std::ops::Range<chrono::NaiveDate>,
   san_ips: &[std::net::IpAddr],
   san_hosts: &[impl AsRef<str>],
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

   Ok((key_pair, cert))
}

pub fn generate_client(
   subject: &str,
   validity: std::ops::Range<chrono::NaiveDate>,
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

   Ok((key_pair, cert))
}
