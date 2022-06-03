use crate::edns::Edns;
use crate::header_flag::HeaderFlag;
use crate::name::Name;
use crate::opcode::Opcode;
use crate::question::Question;
use crate::rcode::Rcode;
use crate::response::{Response, Section, SectionType};
use crate::rr_type::RRType;
use crate::rrset::RRset;
use anyhow::Result;
use std::str::FromStr;

pub struct ResponseBuilder<'a> {
    resp: &'a mut Response,
}

impl<'a> ResponseBuilder<'a> {
    pub fn new(resp: &'a mut Response) -> Self {
        ResponseBuilder { resp }
    }

    pub fn id(&mut self, id: u16) -> &mut Self {
        self.resp.header.id = id;
        self
    }

    pub fn set_flag(&mut self, flag: HeaderFlag) -> &mut Self {
        self.resp.header.set_flag(flag, true);
        self
    }

    pub fn clear_flag(&mut self, flag: HeaderFlag) -> &mut Self {
        self.resp.header.set_flag(flag, false);
        self
    }

    pub fn opcode(&mut self, op: Opcode) -> &mut Self {
        self.resp.header.opcode = op;
        self
    }

    pub fn rcode(&mut self, rcode: Rcode) -> &mut Self {
        self.resp.header.rcode = rcode;
        self
    }

    pub fn question(&mut self, question: Question) -> &mut Self {
        self.resp.question = question;
        self
    }

    pub fn edns(&mut self, ed: Edns) -> &mut Self {
        self.add_rrset(SectionType::Additional, ed.to_rrset());
        self
    }

    pub fn make_response(&mut self) -> &mut Self {
        self.set_flag(HeaderFlag::QueryRespone)
    }

    pub fn add_rrset(&mut self, section: SectionType, mut rrset: RRset) -> &mut Self {
        if let Some(ref mut rrsets) = self.resp.section_mut(section) {
            if let Some(index) = rrsets.iter().position(|old| old.is_same_rrset(&rrset)) {
                rrsets[index].rdatas.append(&mut rrset.rdatas);
            } else {
                rrsets.push(rrset);
            }
        } else {
            self.resp.sections[section as usize] = Section(Some(vec![rrset]));
        }
        self
    }

    pub fn remove_rrset_by<F: FnMut(&RRset) -> bool>(
        &mut self,
        section: SectionType,
        mut f: F,
    ) -> &mut Self {
        if let Some(rrsets) = self.resp.section_mut(section) {
            rrsets.retain(|rrset| !f(rrset));
        }
        self
    }

    pub fn clear_section(&mut self, section: SectionType) -> &mut Self {
        self.resp.clear_section(section);
        self
    }

    pub fn with_section<F: FnOnce(Option<Vec<RRset>>) -> Option<Vec<RRset>>>(
        &mut self,
        section: SectionType,
        f: F,
    ) -> &mut Self {
        let rrsets = self.resp.take_section(section);
        self.resp.sections[section as usize] = Section(f(rrsets));
        self
    }

    pub fn done(&mut self) {
        self.resp.recalculate_header();
    }
}

pub fn build(
    name: &str,
    typ: RRType,
    answers: Vec<Vec<&str>>,
    authorities: Vec<Vec<&str>>,
    additionals: Vec<Vec<&str>>,
    udp_size: Option<usize>,
) -> Result<Response> {
    let mut resp = Response::with_question(Name::from_str(name)?, typ);
    let mut builder = ResponseBuilder::new(&mut resp);
    for rrset in answers {
        builder.add_rrset(SectionType::Answer, RRset::from_strs(rrset.as_slice())?);
    }
    for rrset in authorities {
        builder.add_rrset(SectionType::Authority, RRset::from_strs(rrset.as_slice())?);
    }
    for rrset in additionals {
        builder.add_rrset(SectionType::Additional, RRset::from_strs(rrset.as_slice())?);
    }
    if let Some(udp_size) = udp_size {
        builder.edns(Edns {
            version: 0,
            extened_rcode: 0,
            udp_size: udp_size as u16,
            dnssec_aware: false,
            options: None,
        });
    }
    builder.make_response().done();
    Ok(resp)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::RRType;
    use std::str::FromStr;

    #[test]
    fn test_message_builder() {
        let www_knet_cn_response = vec![
            4, 176, 132, 0, 0, 1, 0, 1, 0, 4, 0, 9, 3, 119, 119, 119, 4, 107, 110, 101, 116, 2, 99,
            110, 0, 0, 1, 0, 1, 192, 12, 0, 1, 0, 1, 0, 0, 1, 44, 0, 4, 202, 173, 11, 42, 192, 16,
            0, 2, 0, 1, 0, 0, 14, 16, 0, 20, 4, 118, 110, 115, 49, 9, 122, 100, 110, 115, 99, 108,
            111, 117, 100, 3, 98, 105, 122, 0, 192, 16, 0, 2, 0, 1, 0, 0, 14, 16, 0, 20, 4, 105,
            110, 115, 49, 9, 122, 100, 110, 115, 99, 108, 111, 117, 100, 3, 99, 111, 109, 0, 192,
            16, 0, 2, 0, 1, 0, 0, 14, 16, 0, 21, 4, 100, 110, 115, 49, 9, 122, 100, 110, 115, 99,
            108, 111, 117, 100, 4, 105, 110, 102, 111, 0, 192, 16, 0, 2, 0, 1, 0, 0, 14, 16, 0, 20,
            4, 99, 110, 115, 49, 9, 122, 100, 110, 115, 99, 108, 111, 117, 100, 3, 110, 101, 116,
            0, 192, 57, 0, 1, 0, 1, 0, 1, 81, 128, 0, 4, 203, 99, 22, 3, 192, 57, 0, 1, 0, 1, 0, 1,
            81, 128, 0, 4, 203, 99, 23, 3, 192, 89, 0, 1, 0, 1, 0, 0, 14, 16, 0, 4, 27, 221, 63, 3,
            192, 89, 0, 1, 0, 1, 0, 0, 14, 16, 0, 4, 119, 167, 244, 44, 192, 121, 0, 1, 0, 1, 0, 0,
            14, 16, 0, 4, 114, 67, 46, 13, 192, 121, 0, 1, 0, 1, 0, 0, 14, 16, 0, 4, 114, 67, 46,
            14, 192, 154, 0, 1, 0, 1, 0, 1, 81, 128, 0, 4, 42, 62, 2, 24, 192, 154, 0, 1, 0, 1, 0,
            1, 81, 128, 0, 4, 42, 62, 2, 29, 0, 0, 41, 16, 0, 0, 0, 0, 0, 0, 0,
        ];
        let mut resp = Response::from_wire(www_knet_cn_response.as_slice()).unwrap();
        let backup = resp.clone();

        let answer = "www.knet.cn.     300     IN      A       202.173.11.42";
        let additional1 = vec![
            "vns1.zdnscloud.biz.	86400	IN	A	203.99.22.3",
            "vns1.zdnscloud.biz.	86400	IN	A	203.99.23.3",
        ];

        let additional2 = vec![
            "ins1.zdnscloud.com.	3600	IN	A	27.221.63.3",
            "ins1.zdnscloud.com.	3600	IN	A	119.167.244.44",
        ];

        let additional3 = vec![
            "dns1.zdnscloud.info.	3600	IN	A	114.67.46.13",
            "dns1.zdnscloud.info.	3600	IN	A	114.67.46.14",
        ];

        let additional4 = vec![
            "cns1.zdnscloud.net.	86400	IN	A	42.62.2.24",
            "cns1.zdnscloud.net.	86400	IN	A	42.62.2.29",
        ];
        assert_eq!(resp.header.ar_count, 9);

        let mut builder = ResponseBuilder::new(&mut resp);
        builder
            .remove_rrset_by(SectionType::Answer, |rrset| rrset.typ == RRType::A)
            .remove_rrset_by(SectionType::Additional, |rrset| rrset.typ == RRType::A)
            .remove_rrset_by(SectionType::Additional, |rrset| rrset.typ == RRType::OPT)
            .done();
        assert_eq!(resp.header.an_count, 0);
        assert_eq!(resp.section_rrset_count(SectionType::Answer), 0);
        assert_eq!(resp.header.ns_count, 4);
        assert_eq!(resp.section_rrset_count(SectionType::Authority), 1);
        assert_eq!(resp.header.ar_count, 0);
        assert_eq!(resp.section_rrset_count(SectionType::Additional), 0);

        let edns = Edns {
            version: 0,
            extened_rcode: 0,
            udp_size: 4096,
            dnssec_aware: false,
            options: None,
        };

        let mut builder = ResponseBuilder::new(&mut resp);
        builder
            .with_section(SectionType::Answer, |_| -> Option<Vec<RRset>> {
                Some(vec![RRset::from_str(answer).unwrap()])
            })
            .add_rrset(
                SectionType::Additional,
                RRset::from_strs(additional1.as_slice()).unwrap(),
            )
            .add_rrset(
                SectionType::Additional,
                RRset::from_strs(additional2.as_slice()).unwrap(),
            )
            .add_rrset(
                SectionType::Additional,
                RRset::from_strs(additional3.as_slice()).unwrap(),
            )
            .add_rrset(
                SectionType::Additional,
                RRset::from_strs(additional4.as_slice()).unwrap(),
            )
            .edns(edns)
            .done();
        assert_eq!(resp, backup);

        let edns = resp.get_edns().unwrap();
        assert_eq!(edns.version, 0);
        assert_eq!(edns.udp_size, 4096);
    }

    #[test]
    fn test_build() {
        let www_knet_cn_response = vec![
            4, 176, 132, 0, 0, 1, 0, 1, 0, 4, 0, 9, 3, 119, 119, 119, 4, 107, 110, 101, 116, 2, 99,
            110, 0, 0, 1, 0, 1, 192, 12, 0, 1, 0, 1, 0, 0, 1, 44, 0, 4, 202, 173, 11, 42, 192, 16,
            0, 2, 0, 1, 0, 0, 14, 16, 0, 20, 4, 118, 110, 115, 49, 9, 122, 100, 110, 115, 99, 108,
            111, 117, 100, 3, 98, 105, 122, 0, 192, 16, 0, 2, 0, 1, 0, 0, 14, 16, 0, 20, 4, 105,
            110, 115, 49, 9, 122, 100, 110, 115, 99, 108, 111, 117, 100, 3, 99, 111, 109, 0, 192,
            16, 0, 2, 0, 1, 0, 0, 14, 16, 0, 21, 4, 100, 110, 115, 49, 9, 122, 100, 110, 115, 99,
            108, 111, 117, 100, 4, 105, 110, 102, 111, 0, 192, 16, 0, 2, 0, 1, 0, 0, 14, 16, 0, 20,
            4, 99, 110, 115, 49, 9, 122, 100, 110, 115, 99, 108, 111, 117, 100, 3, 110, 101, 116,
            0, 192, 57, 0, 1, 0, 1, 0, 1, 81, 128, 0, 4, 203, 99, 22, 3, 192, 57, 0, 1, 0, 1, 0, 1,
            81, 128, 0, 4, 203, 99, 23, 3, 192, 89, 0, 1, 0, 1, 0, 0, 14, 16, 0, 4, 27, 221, 63, 3,
            192, 89, 0, 1, 0, 1, 0, 0, 14, 16, 0, 4, 119, 167, 244, 44, 192, 121, 0, 1, 0, 1, 0, 0,
            14, 16, 0, 4, 114, 67, 46, 13, 192, 121, 0, 1, 0, 1, 0, 0, 14, 16, 0, 4, 114, 67, 46,
            14, 192, 154, 0, 1, 0, 1, 0, 1, 81, 128, 0, 4, 42, 62, 2, 24, 192, 154, 0, 1, 0, 1, 0,
            1, 81, 128, 0, 4, 42, 62, 2, 29, 0, 0, 41, 16, 0, 0, 0, 0, 0, 0, 0,
        ];
        let target = Response::from_wire(www_knet_cn_response.as_slice()).unwrap();

        let qname = "www.knet.cn.";
        let answers = vec![vec![
            "www.knet.cn.    300     IN      A       202.173.11.42",
        ]];
        let authorities = vec![vec![
            "knet.cn.        3600    IN      NS      vns1.zdnscloud.biz.",
            "knet.cn.        3600    IN      NS      ins1.zdnscloud.com.",
            "knet.cn.        3600    IN      NS      dns1.zdnscloud.info.",
            "knet.cn.        3600    IN      NS      cns1.zdnscloud.net.",
        ]];

        let additionals = vec![
            vec![
                "vns1.zdnscloud.biz.     86400   IN      A       203.99.22.3",
                "vns1.zdnscloud.biz.     86400   IN      A       203.99.23.3",
            ],
            vec![
                "ins1.zdnscloud.com.     3600    IN      A       27.221.63.3",
                "ins1.zdnscloud.com.     3600    IN      A       119.167.244.44",
            ],
            vec![
                "dns1.zdnscloud.info.    3600    IN      A       114.67.46.13",
                "dns1.zdnscloud.info.    3600    IN      A       114.67.46.14",
            ],
            vec![
                "cns1.zdnscloud.net.     86400   IN      A       42.62.2.24",
                "cns1.zdnscloud.net.     86400   IN      A       42.62.2.29",
            ],
        ];

        let mut build_msg = build(
            qname,
            RRType::A,
            answers,
            authorities,
            additionals,
            Some(4096),
        )
        .unwrap();
        build_msg.header.id = 1200;
        build_msg.header.set_flag(HeaderFlag::AuthAnswer, true);
        build_msg
            .header
            .set_flag(HeaderFlag::RecursionDesired, false);
        assert_eq!(target, build_msg);
    }
}
