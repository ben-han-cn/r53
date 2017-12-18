use r53::*;

#[test]
fn test_name_concat() {
    let www_knet_cn = Name::new("www.knet.Cn", true).unwrap();
    let www_knet = Name::new("www.knet", true).unwrap();
    let cn = Name::new("cn", true).unwrap();

    println!("<<<after concat {}",
             www_knet.concat(&cn).unwrap().to_string());
    let relation = www_knet_cn.get_relation(&www_knet.concat(&cn).unwrap(), false);
    assert_eq!(relation.order, 0);
    assert_eq!(relation.common_label_count, 4);
    assert_eq!(relation.relation, NameRelation::Equal);

    assert_eq!(www_knet_cn.reverse().to_string(),
               "cn.knet.www.".to_string());

    assert_eq!(www_knet_cn.split(0, 1).unwrap().to_string(),
               "www.".to_string());
    assert_eq!(www_knet_cn.split(0, 4).unwrap().to_string(),
               "www.knet.cn.".to_string());
    assert_eq!(www_knet_cn.split(1, 3).unwrap().to_string(),
               "knet.cn.".to_string());
    assert_eq!(www_knet_cn.split(1, 2).unwrap().to_string(),
               "knet.cn.".to_string());

    assert_eq!(www_knet_cn.parent(0).unwrap().to_string(),
               "www.knet.cn.".to_string());
    assert_eq!(www_knet_cn.parent(1).unwrap().to_string(),
               "knet.cn.".to_string());
    assert_eq!(www_knet_cn.parent(2).unwrap().to_string(),
               "cn.".to_string());
    assert_eq!(www_knet_cn.parent(3).unwrap().to_string(), ".".to_string());
    assert!(www_knet_cn.parent(4).is_err())
}

#[test]
fn test_name_compare() {
    let www_knet_cn_mix_case = Name::new("www.KNET.cN", false).unwrap();
    let www_knet_cn = Name::new("www.knet.cn.", true).unwrap();
    let relation = www_knet_cn.get_relation(&www_knet_cn_mix_case, false);
    assert_eq!(relation.order, 0);
    assert_eq!(relation.common_label_count, 4);
    assert_eq!(relation.relation, NameRelation::Equal);

    let relation = www_knet_cn.get_relation(&www_knet_cn_mix_case, true);
    assert!(relation.order > 0);
    assert_eq!(relation.common_label_count, 1);
    assert_eq!(relation.relation, NameRelation::None);

    let www_knet_com = Name::new("www.knet.com", true).unwrap();
    let relation = www_knet_cn.get_relation(&www_knet_com, false);
    assert!(relation.order < 0);
    assert_eq!(relation.common_label_count, 1);
    assert_eq!(relation.relation, NameRelation::None);

    let baidu_com = Name::new("baidu.com.", true).unwrap();
    let www_baidu_com = Name::new("www.baidu.com", true).unwrap();
    let relation = baidu_com.get_relation(&www_baidu_com, false);
    assert!(relation.order < 0);
    assert_eq!(relation.common_label_count, 3);
    assert_eq!(relation.relation, NameRelation::SuperDomain);
}

#[test]
fn test_name_strip() {
    let www_knet_cn_mix_case = Name::new("www.KNET.cN", true).unwrap();
    assert_eq!(&www_knet_cn_mix_case.strip_left(1).unwrap().to_string(),
               "knet.cn.");
    assert_eq!(&www_knet_cn_mix_case.strip_left(2).unwrap().to_string(),
               "cn.");
    assert_eq!(&www_knet_cn_mix_case.strip_left(3).unwrap().to_string(),
               ".");
    assert_eq!(&www_knet_cn_mix_case.strip_right(1).unwrap().to_string(),
               "www.knet.");
    assert_eq!(&www_knet_cn_mix_case.strip_right(2).unwrap().to_string(),
               "www.");
    assert_eq!(&www_knet_cn_mix_case.strip_right(3).unwrap().to_string(),
               ".");
}

#[test]
fn test_name_hash() {
    let name1 = Name::new("wwwnnnnnnnnnnnnn.KNET.cNNNNNNNNN", false).unwrap();
    let name2 = Name::new("wwwnnnnnnnnnnnnn.KNET.cNNNNNNNNn", false).unwrap();
    let name3 = Name::new("wwwnnnnnnnnnnnnn.KNET.cNNNNNNNNN.baidu.com.cn.net",
                          false).unwrap();
    assert_eq!(&name1.hash(false), &name2.hash(false));
    assert_ne!(&name1.hash(false), &name3.hash(false));
}

#[test]
fn test_name_is_subdomain() {
    let www_knet_cn = Name::new("www.knet.Cn", false ).unwrap();
    let www_knet = Name::new("www.knet", false ).unwrap();
    let knet_cn = Name::new("knet.Cn", false ).unwrap(); 
    let cn = Name::new("cn", false ).unwrap(); 
    let knet = Name::new("kNet", false ).unwrap(); 
    let root = name::root();
    assert!(www_knet_cn.is_subdomain(&knet_cn) &&
            knet_cn.is_subdomain(&cn) &&
            knet_cn.is_subdomain(&root) &&
            cn.is_subdomain(&root) &&
            knet.is_subdomain(&root) &&
            www_knet_cn.is_subdomain(&root) &&
            www_knet.is_subdomain(&root) &&
            root.is_subdomain(&root));
    assert!(knet.is_subdomain(&knet_cn) == false &&
            knet.is_subdomain(&cn) == false &&
            root.is_subdomain(&cn) == false &&
            www_knet.is_subdomain(&www_knet_cn) == false);
}
