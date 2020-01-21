use syn::{
    parse::ParseStream, Attribute, Data, DataStruct, DeriveInput, Error, Fields, FieldsNamed,
    Ident, Lit, Result, Token,
};

pub struct RdataStruct<'a> {
    pub name: &'a Ident,
    pub fields: Vec<Field<'a>>,
}

impl<'a> RdataStruct<'a> {
    pub fn parse_token(node: &'a DeriveInput) -> Result<RdataStruct<'a>> {
        if let Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { ref named, .. }),
            ..
        }) = node.data
        {
            let fields =
                named
                    .iter()
                    .try_fold(Vec::new(), |mut fields, field| -> Result<Vec<Field>> {
                        fields.push(Field::parse_token(field)?);
                        Ok(fields)
                    })?;

            Ok(RdataStruct {
                name: &node.ident,
                fields,
            })
        } else {
            Err(Error::new_spanned(node, "only support struct"))
        }
    }
}

pub struct Field<'a> {
    pub name: &'a Ident,
    pub codec: String,
    pub display: String,
}

impl<'a> Field<'a> {
    pub fn parse_token(field: &'a syn::Field) -> Result<Field<'a>> {
        if field.attrs.len() != 1 {
            return Err(Error::new_spanned(field, "only support one attribute"));
        }

        let attr = field.attrs.iter().next().unwrap();
        if !attr.path.is_ident("field") {
            return Err(Error::new_spanned(attr, "unknown attr path"));
        }

        let (codec, display) = Self::parse_attr(attr)?;
        Ok(Field {
            name: field.ident.as_ref().unwrap(),
            codec,
            display,
        })
    }

    fn parse_attr(attr: &Attribute) -> Result<(String, String)> {
        attr.parse_args_with(|input: ParseStream| {
            let codec: Ident = input.parse()?;
            if codec != "codec" {
                return Err(Error::new_spanned(attr, "no codec"));
            }
            input.parse::<Token![=]>()?;
            let codec_value = if let Lit::Str(ref l) = input.parse::<Lit>()? {
                l.value()
            } else {
                return Err(Error::new_spanned(attr, "codec value isn't string literal"));
            };
            input.parse::<Token![,]>()?;

            let display: Ident = input.parse()?;
            if display != "display" {
                return Err(Error::new_spanned(attr, "no dispaly"));
            }
            input.parse::<Token![=]>()?;
            let display_value = if let Lit::Str(ref l) = input.parse::<Lit>()? {
                l.value()
            } else {
                return Err(Error::new_spanned(attr, "codec value isn't string literal"));
            };
            Ok((codec_value, display_value))
        })
    }
}
