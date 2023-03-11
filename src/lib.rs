extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data::Struct, DeriveInput};

#[proc_macro_attribute]
pub fn table_name(attr: TokenStream, item: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(item);
    let t_name = attr.to_string();
    let output = quote! {
        impl #ident {
            fn table_name() -> String {
                #t_name.into()
            }
        }
    };
    TokenStream::from(output)
}

#[proc_macro_derive(Insertable)]
pub fn insertable(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);
    let mut field_names = Vec::new();
    if let Struct(s) = data {
        for field in s.fields {
            if let Some(f) = field.ident {
                field_names.push(f);
            }
        }
    }
    let column_clause = format!("({})", field_names.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(","));
    let values_clause = format!("({})", field_names.iter().enumerate().map(|(i, _)| format!("${}", (i + 1))).collect::<Vec<String>>().join(","));
    let mut body = quote! {
        let table_name = ident.table_name();
        let mut stmt = format!("INSERT INTO {} ", table_name);
        stmt.push_str(#column_clause);
        stmt.push_str(" VALUES ");
        stmt.push_str(#values_clause);
        stmt.push_str(" RETURN id ");
        let (id, ): (i64, ) = query_as(stmt)
    };
    for f in &field_names {
        body.extend(quote! {
            .bind(&self.#f)
        })
    }
    body.extend(quote! {
        .fetch_one(&mut executor)
        .await?;
        Ok(id)
    });
    TokenStream::from(quote! {
        impl #ident {
            async fn insert<'e, E, D>(&self, executor: E) -> Result<i64, sqlx::Error> where
            E: 'e + sqlx::Executor<'e, Database=sqlx::Postgres>, {
                #body
            }
        }
    })
}
