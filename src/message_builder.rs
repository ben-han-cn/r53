use crate::edns::Edns;
use crate::header_flag::HeaderFlag;
use crate::message::{Message, Section, SectionType};
use crate::opcode::Opcode;
use crate::rcode::Rcode;
use crate::rrset::RRset;

pub struct MessageBuilder<'a> {
    msg: &'a mut Message,
}

impl<'a> MessageBuilder<'a> {
    pub fn new(msg: &'a mut Message) -> Self {
        MessageBuilder { msg }
    }

    pub fn id(&mut self, id: u16) -> &mut Self {
        self.msg.header.id = id;
        self
    }

    pub fn set_flag(&mut self, flag: HeaderFlag) -> &mut Self {
        self.msg.header.set_flag(flag, true);
        self
    }

    pub fn clear_flag(&mut self, flag: HeaderFlag) -> &mut Self {
        self.msg.header.set_flag(flag, false);
        self
    }

    pub fn opcode(&mut self, op: Opcode) -> &mut Self {
        self.msg.header.opcode = op;
        self
    }

    pub fn rcode(&mut self, rcode: Rcode) -> &mut Self {
        self.msg.header.rcode = rcode;
        self
    }

    pub fn edns(&mut self, ed: Edns) -> &mut Self {
        self.msg.edns = Some(ed);
        self
    }

    pub fn make_response(&mut self) -> &mut Self {
        self.set_flag(HeaderFlag::QueryRespone)
    }

    pub fn add_answer(&mut self, rrset: RRset) -> &mut Self {
        self.add_rrset_to_section(SectionType::Answer, rrset)
    }

    pub fn add_auth(&mut self, rrset: RRset) -> &mut Self {
        self.add_rrset_to_section(SectionType::Authority, rrset)
    }

    pub fn add_additional(&mut self, rrset: RRset) -> &mut Self {
        self.add_rrset_to_section(SectionType::Additional, rrset)
    }

    fn add_rrset_to_section(&mut self, section: SectionType, mut rrset: RRset) -> &mut Self {
        if let Some(ref mut rrsets) = self.msg.section_mut(section) {
            if let Some(index) = rrsets.iter().position(|old| old.is_same_rrset(&rrset)) {
                rrsets[index].rdatas.append(&mut rrset.rdatas);
            } else {
                rrsets.push(rrset);
            }
        } else {
            self.msg.sections[section as usize] = Section(Some(vec![rrset]));
        }
        self
    }

    pub fn done(&mut self) {
        self.msg.recalculate_header();
    }
}
