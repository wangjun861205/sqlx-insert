extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data::Struct, DeriveInput};

#[proc_macro_attribute]
pub fn table_name(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut ori = TokenStream2::from(item.clone());
    let DeriveInput { ident, .. } = parse_macro_input!(item);
    let t_name = attr.to_string();
    let output = quote! {
        impl #ident {
            fn table_name(&self) -> String {
                #t_name.into()
            }
        }
    };
    ori.extend(output);
    TokenStream::from(ori)
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
        let table_name = self.table_name();
        let mut stmt = format!("INSERT INTO {} ", table_name);
        stmt.push_str(#column_clause);
        stmt.push_str(" VALUES ");
        stmt.push_str(#values_clause);
        stmt.push_str(" RETURNING id ");
        let (id, ): (i64, ) = sqlx::query_as(&stmt)
    };
    for f in &field_names {
        body.extend(quote! {
            .bind(&self.#f)
        })
    }
    body.extend(quote! {
        .fetch_one(executor)
        .await?;
        Ok(id)
    });
    let func = quote! {
        impl #ident {
            pub async fn sqlx_insert<'e, E>(&self, executor: E) -> Result<i64, sqlx::Error> where
            E: 'e + sqlx::Executor<'e, Database=sqlx::Postgres>, {
                #body
            }
        }
    };
    TokenStream::from(func)
}
