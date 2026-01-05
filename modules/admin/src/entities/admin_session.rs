use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct AdminSessionId(pub [u8; 32]);

#[derive(Debug, Clone, Eq, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct AdminSession {
    pub id: AdminSessionId,
    pub admin_id: Uuid,
}
