fn main() -> std::io::Result<()> {
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(
            &[
                "../../proto/v1/admin/admin-auth.proto",
                "../../proto/v1/common/values.proto",
                "../../proto/v1/auth/admin/user-manage.proto",
                "../../proto/v1/auth/common/account.proto",
                "../../proto/v1/auth/common/email_otp.proto",
                "../../proto/v1/auth/user/account-manage.proto",
                "../../proto/v1/auth/user/auth.proto",
            ],
            &["../../proto"],
        )?;
    Ok(())
}
