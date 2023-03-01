fn main() {
    println!("Run `cargo test`!");
}

#[cfg(test)]
mod test {
    use expect_test::expect_file;
    use indoc::{formatdoc, indoc};
    use quaint::{prelude::Queryable, single::Quaint};
    use sqlparser::{ast::Statement, dialect::PostgreSqlDialect, parser::Parser};

    static PG_URL: &str = "postgresql://postgres:prisma@localhost:5438/postgres";

    #[tokio::test]
    async fn test_all() -> anyhow::Result<()> {
        let quaint = Quaint::new(PG_URL).await?;

        for entry in std::fs::read_dir("schemas")? {
            let entry = entry?;
            let schema_name = entry.file_name().into_string().unwrap();

            println!("Running test for schemas/{schema_name}");

            let schema_setup = formatdoc! {r#"
                DROP SCHEMA IF EXISTS {schema_name} CASCADE;
                CREATE SCHEMA {schema_name};
            "#};

            quaint.raw_cmd(&schema_setup).await?;

            quaint
                .raw_cmd(&format!("SET search_path = \"{schema_name}\""))
                .await?;

            let setup = std::fs::read_to_string(entry.path().join("setup.sql"))?;
            quaint.raw_cmd(&setup).await?;

            let views = get_views(&quaint).await?;
            let view = views.first().unwrap();
            let view_sql = view.parsed.to_string();

            let expected_file = std::env::current_dir()?
                .join(entry.path())
                .join("expected.sql");

            let expected = expect_file![expected_file];

            expected.assert_eq(&view_sql);
        }

        Ok(())
    }

    #[derive(Debug)]
    pub struct View {
        pub name: String,
        pub sql: String,
        pub parsed: Statement,
    }

    async fn get_views(conn: &Quaint) -> anyhow::Result<Vec<View>> {
        let sql = indoc! {r#"
            SELECT viewname AS view_name, definition AS view_sql
            FROM pg_catalog.pg_views
            WHERE schemaname = 'public'
        "#};

        let res = conn.query_raw(sql, &[]).await?;

        let views = res
            .into_iter()
            .map(|row| {
                let name = row.get("view_name").and_then(|v| v.to_string()).unwrap();
                let sql = row.get("view_sql").and_then(|v| v.to_string()).unwrap();
                let parsed = parse_sql(&sql);

                View { name, sql, parsed }
            })
            .collect();

        Ok(views)
    }

    fn parse_sql(sql: &str) -> Statement {
        Parser::new(&PostgreSqlDialect {})
            .try_with_sql(&sql)
            .unwrap()
            .parse_statement()
            .unwrap()
    }
}
