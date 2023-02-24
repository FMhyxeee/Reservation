use std::process::Command;

use tonic_build::Builder;

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        .with_sql_type(&["reservation.ReservationStatus"])
        .with_builder(&[
            "reservation.ReservationQuery",
            "reservation.ReservationFilter",
        ])
        .with_builder_into(
            "reservation.ReservationQuery",
            &[
                "user_id",
                "resource_id",
                "status",
                "page",
                "page_size",
                "desc",
            ],
        )
        .with_builder_into(
            "reservation.ReservationFilter",
            &[
                "user_id",
                "resource_id",
                "status",
                "desc",
                "page_size",
                "cursor",
            ],
        )
        .with_builder_option("reservation.ReservationQuery", &["start", "end"])
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();

    Command::new("cargo").args(["fmt"]).output().unwrap();
    println!("cargo:rerun-if-changed=protos/reservation.proto")
}

trait BuilderExt {
    fn with_sql_type(self, paths: &[&str]) -> Self;
    fn with_builder(self, paths: &[&str]) -> Self;
    fn with_builder_into(self, paths: &str, field: &[&str]) -> Self;
    fn with_builder_option(self, paths: &str, field: &[&str]) -> Self;
}

impl BuilderExt for Builder {
    fn with_sql_type(self, paths: &[&str]) -> Self {
        paths.iter().fold(self, |builder, path| {
            builder.type_attribute(path, "#[derive(sqlx::Type)]")
        })
    }

    fn with_builder(self, paths: &[&str]) -> Self {
        paths.iter().fold(self, |builder, path| {
            builder.type_attribute(path, "#[derive(derive_builder::Builder)]")
        })
    }

    fn with_builder_into(self, paths: &str, field: &[&str]) -> Self {
        field.iter().fold(self, |builder, field| {
            builder.field_attribute(
                format!("{}.{}", paths, field).as_str(),
                "#[builder(setter(into), default)]",
            )
        })
    }

    fn with_builder_option(self, paths: &str, field: &[&str]) -> Self {
        field.iter().fold(self, |builder, field| {
            builder.field_attribute(
                format!("{}.{}", paths, field).as_str(),
                "#[builder(setter(strip_option))]",
            )
        })
    }
}
