use std::collections::HashSet;

#[derive(Hash, PartialOrd, Ord, Eq, PartialEq, Clone)]
pub enum Atom {
    Wildcard,
    Constant(Constant),
    Variable(Variable),
    Tuple(Vec<Atom>),
}

#[derive(PartialOrd, Ord, Clone, Eq, PartialEq, Hash)]
pub struct Constant(pub String);

#[derive(PartialOrd, Ord, Clone, Eq, PartialEq, Hash)]
pub struct Variable(pub String);

pub struct Rule {
    pub consequents: Vec<Atom>,
    pub pos_antecedents: Vec<Atom>,
    // pub checks: Vec<Check>
    pub neg_antecedents: Vec<Atom>,
    // pub checks: Vec<Check>,
    pub diff_sets: Vec<Vec<Atom>>,
    pub same_sets: Vec<Vec<Atom>>,
    pub part_name: Option<Atom>, // ground
}

#[derive(Debug)]
pub struct Program {
    pub rules: Vec<Rule>,
}

impl Atom {
    fn constant(s: &str) -> Self {
        Self::Constant(Constant(s.into()))
    }
    fn visit_atoms<'a: 'b, 'b>(&'a self, visitor: &'b mut impl FnMut(&'a Self)) {
        visitor(self);
        if let Self::Tuple(args) = self {
            for arg in args {
                arg.visit_atoms(visitor)
            }
        }
    }
    pub fn is_tuple(&self) -> bool {
        if let Self::Tuple(_) = self {
            true
        } else {
            false
        }
    }
    pub fn vars_to_wildcards(&self) -> Self {
        match self {
            Self::Constant(Constant(c)) => Self::constant(c),
            Self::Variable(_) | Self::Wildcard => Self::Wildcard,
            Self::Tuple(args) => Self::Tuple(args.iter().map(Self::vars_to_wildcards).collect()),
        }
    }
}

impl Program {
    pub fn preprocess(&mut self) {
        for Rule { consequents, pos_antecedents, .. } in self.rules.iter_mut() {
            // drop consequents that are also antecedents
            consequents.retain(|consequent| !pos_antecedents.contains(consequent))
        }
        // drop rules with no consequents
        self.rules.retain(|rule| !rule.consequents.is_empty());
    }
    pub fn static_reflect_simpler(&mut self) {
        let new_rules: Vec<_> = self
            .rules
            .iter()
            .filter_map(|rule| {
                let name = rule.part_name.as_ref()?;
                Some(Rule {
                    consequents: rule
                        .consequents
                        .iter()
                        .map(|c| {
                            Atom::Tuple(vec![
                                name.clone(),
                                Atom::constant("infers"),
                                c.vars_to_wildcards(),
                            ])
                        })
                        .collect(),
                    pos_antecedents: vec![],
                    neg_antecedents: vec![],
                    diff_sets: vec![],
                    same_sets: vec![],
                    part_name: rule.part_name.clone(),
                })
            })
            .collect();
        self.rules.extend(new_rules);
    }
}

impl Rule {
    pub fn without_neg_antecedents(&self) -> Self {
        Self {
            consequents: self.consequents.clone(),
            pos_antecedents: self.pos_antecedents.clone(),
            diff_sets: self.diff_sets.clone(),
            same_sets: self.same_sets.clone(),
            part_name: self.part_name.clone(),
            neg_antecedents: vec![],
        }
    }
    pub fn unbound_variables<'a, 'b>(&'a self, buf: &'b mut HashSet<&'a Variable>) {
        buf.clear();

        // buffer consequent vars
        let iter = self
            .consequents
            .iter()
            .chain(self.neg_antecedents.iter())
            .chain(self.diff_sets.iter().flat_map(|set| set.iter()))
            .chain(self.same_sets.iter().flat_map(|set| set.iter()));
        for atom in iter {
            atom.visit_atoms(&mut |atom| {
                if let Atom::Variable(var) = atom {
                    buf.insert(var);
                }
            });
        }

        // drop antecedent vars
        for pa in &self.pos_antecedents {
            pa.visit_atoms(&mut |atom| {
                if let Atom::Variable(var) = atom {
                    buf.remove(&var);
                }
            });
        }
    }
}
