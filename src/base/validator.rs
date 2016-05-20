use std::collections::HashMap;
use std::ops::Shl;
use std::any::Any;
use std::mem;
use traitobject;
use urlencoded::QueryResult;
use regex::Regex;
use std::sync::Arc;

#[derive(Clone)]
pub struct Checker {
    field_name: String,
    field_type: Box<FieldType>,
    field_title: String,
    rules: Vec<Rule>,
    optional: bool,
    multiple: bool,
}

pub trait FieldTypeClone {
    fn clone_box(&self) -> Box<FieldType>;
}

impl<T: FieldType + Clone> FieldTypeClone for T {
    fn clone_box(&self) -> Box<FieldType> {
        Box::new(self.clone())
    }
}

impl Clone for Box<FieldType> {
    fn clone(&self) -> Box<FieldType> {
        self.clone_box()
    }
}

pub trait FieldValueParser: Clone {
    fn parse(s: &str) -> Option<Self>;
}

pub trait FieldValue: Any + 'static {
    fn match_rule(&self, rule: &Rule) -> Result<(), String>;
}

impl FieldValue {
    pub fn downcast_ref_unchecked<T: FieldValue>(&self) -> &T {
        unsafe {
            mem::transmute(traitobject::data(self))
        }
    }
}

pub trait FieldType: FieldTypeClone + Any + Send + Sync {
    fn check(&self, checker: &Checker, raw: String) -> Result<Option<Box<FieldValue>>, String>;
}

pub trait FieldTypeWithValue {
    type Value: FieldValueParser + FieldValue;

    fn check(&self, checker: &Checker, raw: String) -> Result<Option<Self::Value>, String> {
        let value = Self::Value::parse(&raw);
        if value.is_none() {
            return Err(format!("{}格式有误", checker.field_title));
        }
        self.check_value(checker, value.unwrap()).map(|v| Some(v))
    }

    fn check_value(&self, checker: &Checker, value: Self::Value) -> Result<Self::Value, String> {
        let mut lambda: Option<Box<Arc<Fn(Box<FieldValue>) -> bool>>> = None;
        for rule in &checker.rules {
            match rule.clone() {
                Lambda(l) => {
                    lambda = Some(l);
                    continue;
                },
                _ => {},
            }
            if let Err(msg) = value.match_rule(rule) {
                return Err(format!("{}{}", checker.field_title, msg));
            }
        }

        if lambda.is_some() {
            let f = *lambda.unwrap();
            if !f(Box::new(value.clone())) {
                return Err(format!("{}{}", checker.field_title, "格式不正确"))
            }
        }

        Ok(value)
    }
}

impl <A: FieldValueParser + FieldValue, B: FieldTypeWithValue<Value=A> + Clone + Any + Send + Sync> FieldType for B {
    fn check(&self, checker: &Checker, raw: String) -> Result<Option<Box<FieldValue>>, String> {
        self.check(checker, raw).map(|v| {
            v.map(|v| Box::new(v) as Box<FieldValue>)
        })
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum Rule {
    Max(i64),
    Min(i64),
    Format(&'static str),
    Lambda(Box<Arc<Fn(Box<FieldValue>) -> bool>>),
    Optional,
    Multiple,
}

pub use self::Rule::*;

impl Checker {
    pub fn new<T>(field_name: &str, field_type: T, field_title: &str) -> Checker
        where T: FieldTypeWithValue + Clone + Any + Send + Sync {

        Checker{
            field_name: field_name.to_string(),
            field_type: Box::new(field_type),
            field_title: field_title.to_string(),
            rules: Vec::new(),
            optional: false,
            multiple: false,
        }
    }

    fn check_multiple(&self, raw: Option<Vec<String>>) -> Result<Option<Vec<Box<FieldValue>>>, String> {
        let mut empty_or_none = false;
        let mut values: Vec<String> = Vec::new();
        if raw.is_none() {
            empty_or_none = true;
        } else {
            values = raw.unwrap();
            if values.len() < 1 {
                empty_or_none = true;
            }
        }

        if empty_or_none {
            if self.optional {
                return Ok(None);
            } else {
                return Err(format!("{}不能为空", self.field_name));
            }
        }

        let mut field_values = Vec::new();
        for value in values {
            match self.check(Some(value)) {
                Ok(Some(field_value)) => {
                    field_values.push(field_value);
                },
                Err(e) => {
                    return Err(e);
                },
                _ => {},
            }
        }
        Ok(Some(field_values))
    }

    fn check(&self, raw: Option<String>) -> Result<Option<Box<FieldValue>>, String> {
        let mut empty_or_none = false;
        let mut value: String = String::new();
        if raw.is_none() {
            empty_or_none = true;
        } else {
            value = raw.unwrap();
            if value.len() < 1 {
                empty_or_none = true;
            }
        }

        if empty_or_none {
            if self.optional {
                return Ok(None);
            } else {
                return Err(format!("{}不能为空", self.field_title));
            }
        }

        self.field_type.check(self, value)
    }

}

impl Shl<Rule> for Checker {

    type Output = Checker;

    fn shl(self, rule: Rule) -> Checker {
        let mut checker = self.clone();

        match rule {
            Optional => checker.optional = true,
            Multiple => checker.multiple = true,
            _ => checker.rules.push(rule),
        }

        checker
    }
}

pub struct Validator {
    checkers: Vec<Checker>,
    pub valid_data: HashMap<String, Vec<Box<FieldValue>>>,
    pub messages: Vec<String>,
}

impl Validator {
    pub fn new() -> Validator {
        Validator {
            checkers: Vec::new(),
            valid_data: HashMap::new(),
            messages: Vec::new(),
        }
    }

    pub fn get_valid<T: FieldValue>(&self, name: &str) -> &T {
        self.valid_data.get(name).unwrap()[0].downcast_ref_unchecked::<T>()
    }

    pub fn add_checker(&mut self, checker: Checker) -> &mut Self {
        self.checkers.push(checker);
        self
    }

    pub fn validate(&mut self, query: QueryResult) {
        // 如果没有checker，则当做有效
        if self.checkers.len() < 1 {
            return;
        }

        match query {
            Ok(query_map) => {
                for checker in &self.checkers {
                    let (multiple_values, value) = match query_map.get(&checker.field_name) {
                        Some(values) => (Some(values.clone()), Some(values[0].clone())),
                        None => (None::<Vec<String>>,  None::<String>)
                    };

                    if checker.multiple {
                        match checker.check_multiple(multiple_values) {
                            Ok(Some(values)) => {
                                self.valid_data.insert(checker.field_name.clone(), values);
                            },
                            Err(e) => {
                                self.messages.push(e);
                            },
                            _ => {},
                        }
                    } else {
                        match checker.check(value) {
                            Ok(Some(field_value)) => {
                                self.valid_data.insert(checker.field_name.clone(), vec![field_value]);
                            },
                            Err(e) => {
                                self.messages.push(e);
                            },
                            _ => {},
                        }
                    }

                }
            },
            Err(_) => {
                self.messages.push("参数不合法".to_string());
            },
        }
    }

    pub fn is_valid(&self) -> bool {
        self.messages.len() < 1
    }
}

// ********* str checker ***********

#[derive(Clone)]
pub struct Str;

impl FieldTypeWithValue for Str {
    type Value = StrValue;
}

#[derive(Clone)]
pub struct StrValue(String);

impl StrValue {
    pub fn value(&self) -> String {
        self.0.clone()
    }
}

impl FieldValueParser for StrValue {
    fn parse(s: &str) -> Option<Self> {
        Some(StrValue(s.clone().to_string()))
    }
}

impl FieldValue for StrValue {
    fn match_rule(&self, rule: &Rule) -> Result<(), String> {
        match *rule {
            Max(max) => {
                if self.0.len() > max as usize {
                    return Err(format!("长度不能大于{}", max));
                }
            },
            Min(min) => {
                if self.0.len() < min as usize {
                    return Err(format!("长度不能小于{}", min));
                }
            },
            Format(format) => {
                let re = Regex::new(&format).unwrap();
                if !re.is_match(&self.0) {
                    return Err(format!("格式不正确"));
                }
            },
            _ => {},
        }
        Ok(())
    }
}

// ********* int checker ***********
#[derive(Clone)]
pub struct Int;

impl FieldTypeWithValue for Int {
    type Value = IntValue;
}

#[derive(Clone)]
pub struct IntValue(i64);

impl IntValue {
    pub fn value(&self) -> i64 {
        self.0
    }
}

impl FieldValueParser for IntValue {
    fn parse(s: &str) -> Option<Self> {
        s.parse::<i64>().map(|v| IntValue(v)).ok()
    }
}

impl FieldValue for IntValue {
    fn match_rule(&self, rule: &Rule) -> Result<(), String> {
        match *rule {
            Max(max) => {
                if self.0 > max {
                    return Err(format!("不能大于{}", max));
                }
            },
            Min(min) => {
                if self.0 < min {
                    return Err(format!("不能小于{}", min));
                }
            },
            _ => {},
        }
        Ok(())
    }
}
