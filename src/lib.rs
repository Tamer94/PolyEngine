use std::collections::{HashSet, HashMap};
use std::fmt::Display;
use std::hash::Hash;
use std::num::{NonZeroI64};
use std::ops::{Add, Mul, Sub, Index, AddAssign};
use array_init::array_init;

#[derive(Clone, Copy, Debug)]
pub struct Monom {
    var_product: u128,
    factor: NonZeroI64,
}

impl Add<i64> for Monom {
    type Output = Option<Monom>;

    fn add(mut self, rhs: i64) -> Option<Monom> {
        let new_factor = self.factor.get() + rhs;
        if new_factor != 0 {
            self.factor = NonZeroI64::new(new_factor).unwrap();
            Some(self)
        } else {
            None
        }
    }
}

impl Sub<i64> for Monom {
    type Output = Option<Monom>;

    fn sub(mut self, rhs: i64) -> Option<Monom> {
        let new_factor = self.factor.get() - rhs;
        if new_factor != 0 {
            self.factor = NonZeroI64::new(new_factor).unwrap();
            Some(self)
        } else {
            None
        }
    }
}

impl Mul<i64> for Monom {
    type Output = Option<Monom>;

    fn mul(mut self, rhs: i64) -> Option<Monom> {
        let new_factor = self.factor.get() * rhs;
        if new_factor != 0 {
            self.factor = NonZeroI64::new(new_factor).unwrap();
            Some(self)
        } else {
            None
        }
    }
}

impl Mul<Monom> for Monom {
    type Output = Self;

    fn mul(mut self, rhs: Monom) -> Self::Output {
        self.factor = NonZeroI64::new(self.factor.get() * rhs.factor.get()).unwrap();
        self.var_product = self.var_product | rhs.var_product;
        self
    }
}

// impl<const N: usize> From<(i64, [u32; N])> for Monom {
//     fn from(contents: (i64, [u32; N])) -> Self {
//         let mut bitset = 0;
//         for i in contents.1 {
//             let set_bit = 1 << i;
//             bitset |= set_bit;
//         }
// 
//         Monom { var_product: bitset, factor: NonZeroI64::new(contents.0).unwrap() }
//     }
// }

impl<C: IntoIterator<Item = u32>> From<(i64, C)> for Monom {
    fn from(contents: (i64, C)) -> Self {
        let mut bitset = 0;
        for i in contents.1 {
            let set_bit = 1 << i;
            bitset |= set_bit;
        }

        Monom { var_product: bitset, factor: NonZeroI64::new(contents.0).unwrap() }
    }
}

impl PartialEq for Monom {
    fn eq(&self, other: &Self) -> bool {
        self.var_product == other.var_product
        
    }
}

impl Eq for Monom {
}

impl Hash for Monom {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.var_product.hash(state);
    }
}

impl PartialOrd for Monom {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.factor.get().abs().cmp(&other.factor.get().abs()))
    }
}

impl Ord for Monom {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.factor.get().abs().cmp(&other.factor.get().abs())
    }
}


impl Monom {
    pub fn all_used_vars(&self) -> Vec<usize> {
        let mut used = vec![];
        let mut vars = self.var_product;
        let mut index = u128::BITS as usize;
        loop {
            if vars == 0 {
                break;
            }
            index -= 1;
            let clz = vars.leading_zeros();
            index -= clz as usize;

            used.push(index);
            (vars, _) = vars.overflowing_shl(clz + 1);


            if vars <= 1 {
                break;
            }

        }

        used
    }

    pub fn all_free_vars(&self) -> ([bool; 128]) {
        let mut free = [false; 128];
        let mut vars = self.var_product;
        let mut index = u128::BITS as usize;
        loop {
            if vars == u128::MAX {
                break;
            }
            index -= 1;
            let clo = vars.leading_ones();
            index -= clo as usize;

            free[index] = true;
            (vars, _) = vars.overflowing_shl(clo + 1);


            if index < 1 {
                break;
            }

        }

        free
    }


    pub fn to_string<C>(&self, var_names: &C) -> String 
    where
        C: IntoIterator + Index<usize>,
        <C as Index<usize>>::Output: Display + Sized,
    {
        let mut output = format!("{:+}·", self.factor.get());
        let mut vars = self.var_product;
        let mut index = u128::BITS as usize;
        loop {
            if vars == 0 {
                break;
            }
            // println!("vars before left shift {}", vars);
            // println!("clz: {}", vars.leading_zeros());
            index -= 1;
            let clz = vars.leading_zeros();
            index -= clz as usize;
            // println!("{index}, {:X}", vars);
            output.push_str(&format!("{}·", var_names[index]));
            (vars, _) = vars.overflowing_shl(clz + 1);
            // println!("vars after left shift {}", vars);
            // println!("vars second after left shift {}", vars);
            // println!("2: clz: {}", vars.leading_zeros());

            if vars <= 1 {
                break;
            }
            // println!("3: clz: {}", vars.leading_zeros());
        }
        output.pop();
        output
    }

    pub fn cmp_vars(&self, other: &Self) -> std::cmp::Ordering {
        self.var_product.cmp(&other.var_product)
    }

    pub fn delete_var(&mut self, var: u32) {
        self.var_product &= !(1_u128 << var);
    }

    pub fn set_var(&mut self, var: u32) {
        self.var_product |= 1_u128 << var;
    }
}

#[derive(Debug, Clone)]
pub struct BPolynom {
    pub poly: HashSet<Monom>,
}

impl Add<Monom> for BPolynom {
    type Output = BPolynom;

    fn add(mut self, rhs: Monom) -> Self::Output {
        if let Some(&m) = self.poly.get(&rhs) {
            let result = m + rhs.factor.get();
            self.poly.remove(&rhs);
            if let Some(m) = result {
                self.poly.insert(m);
            }
        } else {
            self.poly.insert(rhs);
        }
        self
    }
}

impl Add<&Monom> for &mut BPolynom {
    type Output = ();

    fn add(self, rhs: &Monom) -> Self::Output {
        if let Some(&m) = self.poly.get(rhs) {
            let result = m + rhs.factor.get();
            self.poly.remove(&rhs);
            if let Some(m) = result {
                self.poly.insert(m);
            }
        } else {
            self.poly.insert(*rhs);
        }
        ()
    }
}

impl AddAssign<&Monom> for BPolynom {

    fn add_assign(&mut self, rhs: &Monom) {
        if let Some(&m) = self.poly.get(rhs) {
            let result = m + rhs.factor.get();
            self.poly.remove(&rhs);
            if let Some(m) = result {
                self.poly.insert(m);
            }
        } else {
            self.poly.insert(*rhs);
        }
    }
}

impl Add<&BPolynom> for BPolynom {
    type Output = Self;

    fn add(mut self, rhs: &BPolynom) -> Self::Output {
        for m in rhs.poly.iter() {
            self = self + *m;
        }
        self
    }
}

impl Mul<i64> for BPolynom {
    type Output = BPolynom;

    fn mul(mut self, rhs: i64) -> Self::Output {
        let vec_monoms: Vec<Monom> = self.poly.iter().map(|x| *x).collect();
        for m in vec_monoms {
            self.poly.remove(&m);
            let v = m * rhs;
            if let Some(m1) = v {
                self.poly.insert(m1);
            }
        }
        self
    }
}


impl Mul<Monom> for BPolynom {
    type Output = BPolynom;

    fn mul(mut self, rhs: Monom) -> Self::Output {
        let vec_monoms: Vec<Monom> = self.poly.iter().map(|x| *x).collect();
        for m in vec_monoms {
            self.poly.remove(&m);
            let v = m * rhs;
            self.poly.insert(v);
        }
        self
    }
}

impl Mul<&BPolynom> for &BPolynom {
    type Output = BPolynom;

    fn mul(self, rhs: &BPolynom) -> Self::Output {
        let mut product_poly = BPolynom::empty();
        for &m in rhs.poly.iter() {
            let partial = self.clone() * m;
            product_poly = product_poly + &partial;
        }
        product_poly
    }
}

impl<O, I> From<O> for BPolynom
where
    O: IntoIterator<Item = (i64, I)>,
    I: IntoIterator<Item = u32>,
{
    fn from(container: O) -> Self {
        let mut new_poly = BPolynom {
            poly: HashSet::new(),
        };
        for tuple in container {
            let m = Monom::from(tuple);
            new_poly = new_poly + m;
        }
        new_poly
    }
}

impl BPolynom {
    pub fn empty() -> Self {
        BPolynom { poly: HashSet::new() }
    }

    pub fn to_string<C>(&self, var_names: &C, seperator: &str) -> String 
    where
        C: IntoIterator + Index<usize>,
        <C as Index<usize>>::Output: Display + Sized,
    {
        let mut output = String::from("");
        let mut sorted_vec: Vec<Monom> = self.poly.iter().map(|x|*x).collect();
        sorted_vec.sort_by(|a, b| b.factor.get().abs().cmp(&a.factor.get().abs()));
        for m in sorted_vec {
            output.push_str(&m.to_string(var_names));
            output.push_str(seperator);
        }
        if output.len() > 0 {
            output.truncate(output.len() - seperator.len());
        } else {
            output = String::from("0");
        }
        output
    }
}

#[derive(Debug, Clone)]
pub struct PolyEngine {
    pub p: BPolynom,
    pub var_names: [String; 128],
    pub var_mapping: [usize; 128],
    pub reverse_mapping: HashMap<usize, u32>,
    pub var_occurences: [HashSet<Monom>; 128],
    pub free_var_slots: [bool; 128],
}

impl PolyEngine {
    pub fn new(p: BPolynom) -> Self {
        let mut new_engine = PolyEngine {
            p,
            var_names: array_init(|_| String::new()),
            var_mapping: [0; 128],
            reverse_mapping: HashMap::new(),
            var_occurences: array_init(|_| HashSet::new()),
            free_var_slots: [true; 128],
        };
        let mut bitset = 0;
        for m in new_engine.p.poly.iter() {
            bitset |= m.var_product;
            for index in m.all_used_vars() {
                new_engine.var_occurences[index].insert(*m);
            }
        }
        new_engine.free_var_slots = (Monom{ factor: NonZeroI64::new(1).unwrap(), var_product: bitset }).all_free_vars();

        new_engine
    }

    pub fn add_from_generates(&mut self, poly: BPolynom) {
        self.p = poly;
        let mut bitset = 0;
        for m in self.p.poly.iter() {
            bitset |= m.var_product;
            for index in m.all_used_vars() {
                self.var_occurences[index].insert(*m);
            }
        }
    } 

    pub fn next_free_var(&self) -> Option<usize> {
        self.free_var_slots.iter().position(|&x| x == true)
    }
    
    pub fn free_var(&mut self, var: usize) -> HashSet<Monom> {
        let mapped = self.var_mapping[var];
        self.reverse_mapping.remove(&mapped);
        self.var_names[var] = String::new();
        let output = self.var_occurences[var].clone();
        self.var_occurences[var].clear();
        output
    }

    pub fn get_2_compl_poly(&mut self, vars: Vec<usize>, names: Vec<String>) -> BPolynom {
        let mut new_poly = BPolynom::empty();
        let mut factor = 1;
        for i in 0..vars.len() {
            if i == vars.len() - 1 {
                factor *= -1;
            }
            if let Some(pos) = self.next_free_var() {
                self.free_var_slots[pos] = false;
                self.var_names[pos] = names[i].clone();
                self.var_mapping[pos] = vars[i];
                self.reverse_mapping.insert(vars[i], pos as u32);
                let new_monom = Monom::from((factor, [pos as u32]));
                // self.var_occurences[pos].push(new_monom);
                // println!("{}", new_monom.to_string(&self.var_names));
                new_poly = new_poly + new_monom;
            }
            factor *= 2;
        }
        new_poly
    }

    pub fn get_unsigned_poly(&mut self, vars: Vec<usize>, names: Vec<String>) -> BPolynom {
        let mut new_poly = BPolynom::empty();
        let mut factor = 1;
        for i in 0..vars.len() {
            if let Some(pos) = self.next_free_var() {
                self.free_var_slots[pos] = false;
                self.var_names[pos] = names[i].clone();
                self.var_mapping[pos] = vars[i];
                self.reverse_mapping.insert(vars[i], pos as u32);
                let new_monom = Monom::from((factor, [pos as u32]));
                // self.var_occurences[pos].push(new_monom);
                new_poly = new_poly + new_monom;
            }
            factor *= 2;
        }
        new_poly
    }

    pub fn const_1_replace(&mut self, out: usize,) {
        let &monom_var = self.reverse_mapping.get(&out).unwrap();
        let old_var_name = self.var_names[monom_var as usize].clone();
        let occurences = self.free_var(monom_var as usize);
        self.free_var_slots[monom_var as usize] = true;
        self.var_occurences[monom_var as usize].clear();
        println!("replace {} with 1", old_var_name);

        
        for mut m in occurences {
            let _removed = self.p.poly.remove(&m);
            // println!("removed? {}!", removed);

            for var in m.all_used_vars() {
                self.var_occurences[var].remove(&m);
            }
            m.delete_var(monom_var);
            let all_used_vars = m.all_used_vars();
            for &var in &all_used_vars {
                self.var_occurences[var].remove(&m);
            }
            self.p += &m;
            if self.p.poly.contains(&m) {
                for &var in &all_used_vars {
                    self.var_occurences[var].insert(m);
                }
            }
        }
    }

    pub fn const_0_replace(&mut self, out: usize,) {
        let &monom_var = self.reverse_mapping.get(&out).unwrap();
        let old_var_name = self.var_names[monom_var as usize].clone();
        let occurences = self.free_var(monom_var as usize);
        self.free_var_slots[monom_var as usize] = true;
        self.var_occurences[monom_var as usize].clear();
        println!("replace {} with 0", old_var_name);

        
        for m in occurences {
            let _removed = self.p.poly.remove(&m);
            // println!("removed? {}!", removed);

            for var in m.all_used_vars() {
                self.var_occurences[var].remove(&m);
            }
        }
    }

    pub fn not_replace(&mut self, out: usize, in1: usize, in_name: String) {
        let &monom_var = self.reverse_mapping.get(&out).unwrap();
        let old_var_name = self.var_names[monom_var as usize].clone();
        let occurences = self.free_var(monom_var as usize);
        self.free_var_slots[monom_var as usize] = true;
        self.var_occurences[monom_var as usize].clear();
        let replacement_var = match self.reverse_mapping.get(&in1) {
            Some(&v) => v,
            None => {
                let new_var = self.next_free_var().unwrap();
                self.free_var_slots[new_var] = false;
                self.var_names[new_var] = in_name;
                self.var_mapping[new_var] = in1;
                self.reverse_mapping.insert(in1, new_var as u32);
                new_var as u32
            }
        };

        println!("replace {} with ¬{}", old_var_name, self.var_names[replacement_var as usize]);

        
        for mut m in occurences {
            let _removed = self.p.poly.remove(&m);
            // println!("removed? {}!", removed);

            for var in m.all_used_vars() {
                self.var_occurences[var].remove(&m);
            }
            let m_copy = m;
            m.delete_var(monom_var);
            let all_used_vars = m.all_used_vars();
            for &var in &all_used_vars {
                self.var_occurences[var].remove(&m);
            }
            self.p += &m;
            if let Some(entry) = self.p.poly.get(&m) {
                for &var in &all_used_vars {
                    self.var_occurences[var].insert(*entry);
                }
            }
            m.set_var(replacement_var);
            m = (m * -1).unwrap();
            for var in m.all_used_vars() {
                self.var_occurences[var].remove(&m);
            }
            self.p += &m;
            if let Some(entry) = self.p.poly.get(&m) {
                for &var in &all_used_vars {
                    self.var_occurences[var].insert(*entry);
                }
                self.var_occurences[replacement_var as usize].insert(*entry);
            }
            if !self.p.poly.contains(&m_copy) {
                for var in all_used_vars {
                    self.var_occurences[var].remove(&m_copy);
                }   
            }
        }
    }

    pub fn xor_replace(&mut self, out: usize, in1: usize, in_name1: String, in2: usize, in_name2: String) {
        let &monom_var = self.reverse_mapping.get(&out).unwrap();
        let old_var_name = self.var_names[monom_var as usize].clone();
        let occurences = self.var_occurences[monom_var as usize].clone();
        let replacement_var1 = match self.reverse_mapping.get(&in1) {
            Some(&v) => v,
            None => {
                let new_var = self.next_free_var().unwrap();
                self.free_var_slots[new_var] = false;
                self.var_names[new_var] = in_name1;
                self.var_mapping[new_var] = in1;
                self.reverse_mapping.insert(in1, new_var as u32);
                new_var as u32
            }
        };

        let replacement_var2 = match self.reverse_mapping.get(&in2) {
            Some(&v) => v,
            None => {
                let new_var = self.next_free_var().unwrap();
                self.free_var_slots[new_var] = false;
                self.var_names[new_var] = in_name2;
                self.var_mapping[new_var] = in2;
                self.reverse_mapping.insert(in2, new_var as u32);
                new_var as u32
            }
        };

        println!("replace {} with {}⨁ {}", old_var_name, self.var_names[replacement_var1 as usize], self.var_names[replacement_var2 as usize]);

        
        for mut m in occurences {
            let _removed = self.p.poly.remove(&m);
            println!("monom {} removed!", m.to_string(&self.var_names));
            for var in m.all_used_vars() {
                self.var_occurences[var].remove(&m);
            }
            println!("\x1B[32m");
            self.print_var_occurences();
            println!("\x1B[0m");

            m.delete_var(monom_var);
            let all_used_vars = m.all_used_vars();
            let mut new_monom2 = m.clone();
            m.set_var(replacement_var1);
            new_monom2.set_var(replacement_var2);
            for &var in &all_used_vars {
                self.var_occurences[var].remove(&m);
            }
            println!("\x1B[32m");
            println!("Remove {}", m.to_string(&self.var_names));
            self.print_var_occurences();
            println!("\x1B[0m");
            for var in new_monom2.all_used_vars() {
                self.var_occurences[var].remove(&new_monom2);
            }
            println!("\x1B[32m");
            println!("Remove {}", new_monom2.to_string(&self.var_names));
            self.print_var_occurences();
            println!("\x1B[0m");
            
            self.p += &m;
            if let Some(entry) = self.p.poly.get(&m) {
                for &var in &all_used_vars {
                    self.var_occurences[var].insert(*entry);
                }
                self.var_occurences[replacement_var1 as usize].insert(*entry);
            }
            println!("\x1B[32m");
            println!("Added {}", m.to_string(&self.var_names));
            self.print_var_occurences();
            println!("\x1B[0m");


            self.p += &new_monom2;
            if let Some(entry) = self.p.poly.get(&new_monom2) {
                for &var in &all_used_vars {
                    self.var_occurences[var].insert(*entry);
                }
                self.var_occurences[replacement_var2 as usize].insert(*entry);
            }
            println!("\x1B[32m");
            println!("Added {}", new_monom2.to_string(&self.var_names));
            self.print_var_occurences();
            println!("\x1B[0m");

            new_monom2.set_var(replacement_var1);
            new_monom2 = (new_monom2 * -2).unwrap();
            for var in new_monom2.all_used_vars() {
                self.var_occurences[var].remove(&new_monom2);
            }
            println!("\x1B[32m");
            println!("Remove {}", new_monom2.to_string(&self.var_names));
            self.print_var_occurences();
            println!("\x1B[0m");
            self.p += &new_monom2;


            if let Some(entry) = self.p.poly.get(&new_monom2) {
                for &var in &all_used_vars {
                    self.var_occurences[var].insert(*entry);
                }
                self.var_occurences[replacement_var1 as usize].insert(*entry);
                self.var_occurences[replacement_var2 as usize].insert(*entry);
            }
            println!("\x1B[32m");
            println!("Added {}", new_monom2.to_string(&self.var_names));
            self.print_var_occurences();
            println!("\x1B[0m");



        }
        let occurences = self.free_var(monom_var as usize);
        self.free_var_slots[monom_var as usize] = true;
        self.var_occurences[monom_var as usize].clear();
    }

    pub fn or_replace(&mut self, out: usize, in1: usize, in_name1: String, in2: usize, in_name2: String) {
        let &monom_var = self.reverse_mapping.get(&out).unwrap();
        let old_var_name = self.var_names[monom_var as usize].clone();
        let occurences = self.free_var(monom_var as usize);
        self.free_var_slots[monom_var as usize] = true;
        self.var_occurences[monom_var as usize].clear();
        let replacement_var1 = match self.reverse_mapping.get(&in1) {
            Some(&v) => v,
            None => {
                let new_var = self.next_free_var().unwrap();
                self.free_var_slots[new_var] = false;
                self.var_names[new_var] = in_name1;
                self.var_mapping[new_var] = in1;
                self.reverse_mapping.insert(in1, new_var as u32);
                new_var as u32
            }
        };

        let replacement_var2 = match self.reverse_mapping.get(&in2) {
            Some(&v) => v,
            None => {
                let new_var = self.next_free_var().unwrap();
                self.free_var_slots[new_var] = false;
                self.var_names[new_var] = in_name2;
                self.var_mapping[new_var] = in2;
                self.reverse_mapping.insert(in2, new_var as u32);
                new_var as u32
            }
        };

        println!("replace {} with {}∨{}", old_var_name, self.var_names[replacement_var1 as usize], self.var_names[replacement_var2 as usize]);
        
        for mut m in occurences {
            let _removed = self.p.poly.remove(&m);
            // println!("removed? {}!", removed);
            for var in m.all_used_vars() {
                self.var_occurences[var].remove(&m);
            } 
            m.delete_var(monom_var);
            let all_used_vars = m.all_used_vars();
            let mut new_monom2 = m.clone();
            m.set_var(replacement_var1);
            new_monom2.set_var(replacement_var2);
            for &var in &all_used_vars {
                self.var_occurences[var].remove(&m);
            }
            for var in new_monom2.all_used_vars() {
                self.var_occurences[var].remove(&new_monom2);
            }
            
            self.p += &m;
            if let Some(entry) = self.p.poly.get(&m) {
                for &var in &all_used_vars {
                    self.var_occurences[var].insert(*entry);
                }
                self.var_occurences[replacement_var1 as usize].insert(*entry);
            }


            self.p += &new_monom2;
            if let Some(entry) = self.p.poly.get(&new_monom2) {
                for &var in &all_used_vars {
                    self.var_occurences[var].insert(*entry);
                }
                self.var_occurences[replacement_var2 as usize].insert(*entry);
            }

            new_monom2.set_var(replacement_var1);
            new_monom2 = (new_monom2 * -1).unwrap();
            for var in new_monom2.all_used_vars() {
                self.var_occurences[var].remove(&new_monom2);
            }
            self.p += &new_monom2;


            if let Some(entry) = self.p.poly.get(&new_monom2) {
                for &var in &all_used_vars {
                    self.var_occurences[var].insert(*entry);
                }
                self.var_occurences[replacement_var1 as usize].insert(*entry);
                self.var_occurences[replacement_var2 as usize].insert(*entry);
            }

        }
    }

    pub fn and_replace(&mut self, out: usize, in1: usize, in_name1: String, in2: usize, in_name2: String) {
        let &monom_var = self.reverse_mapping.get(&out).unwrap();
        let old_var_name = self.var_names[monom_var as usize].clone();
        let occurences = self.free_var(monom_var as usize);
        self.free_var_slots[monom_var as usize] = true;
        self.var_occurences[monom_var as usize].clear();
        let replacement_var1 = match self.reverse_mapping.get(&in1) {
            Some(&v) => v,
            None => {
                let new_var = self.next_free_var().unwrap();
                self.free_var_slots[new_var] = false;
                self.var_names[new_var] = in_name1;
                self.var_mapping[new_var] = in1;
                self.reverse_mapping.insert(in1, new_var as u32);
                new_var as u32
            }
        };

        let replacement_var2 = match self.reverse_mapping.get(&in2) {
            Some(&v) => v,
            None => {
                let new_var = self.next_free_var().unwrap();
                self.free_var_slots[new_var] = false;
                self.var_names[new_var] = in_name2;
                self.var_mapping[new_var] = in2;
                self.reverse_mapping.insert(in2, new_var as u32);
                new_var as u32
            }
        };

        println!("replace {} with {}·{}", old_var_name, self.var_names[replacement_var1 as usize], self.var_names[replacement_var2 as usize]);
        
        for mut m in occurences {
            let _removed = self.p.poly.remove(&m);
            for var in m.all_used_vars() {
                self.var_occurences[var].remove(&m);
            }
            // println!("removed? {}!", removed);
            let m_copy = m;
            m.delete_var(monom_var);
            let all_used_vars = m.all_used_vars();
            m.set_var(replacement_var1);
            m.set_var(replacement_var2);
            for var in m.all_used_vars() {
                self.var_occurences[var].remove(&m);
            }
            self.p += &m;
            if let Some(entry) = self.p.poly.get(&m) {
                for &var in &all_used_vars {
                    self.var_occurences[var].insert(*entry);
                }
                self.var_occurences[replacement_var1 as usize].insert(*entry);
                self.var_occurences[replacement_var2 as usize].insert(*entry);
            }
            if !self.p.poly.contains(&m_copy) {
                for var in all_used_vars {
                    self.var_occurences[var].remove(&m_copy);
                }   
            }
        }
    }

    pub fn print_var_occurences(&self) {
        println!("\n\x1B[31m--- var occurences\x1B[0m");
        for (i, list) in self.var_occurences.iter().enumerate() {
            if list.len() > 0 { print!("{}: ", self.var_names[i]); }
            for o in list {
                print!("{}, ", o.to_string(&self.var_names));
            }
            
            if list.len() > 0 { print!("\n"); }
        }
        println!("\n --- end var occurences");
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::format};

    use super::*;

    #[test]
    fn it_works() {
        let var_names = ["q0", "_1_", "_2_", "z", "z_1", "t0", "t1", "t3", "t4"];
        let monom = Monom::from((55, [0, 1, 3, 2, 8]));
        let monom = (monom + -56).unwrap();
        println!("{}", monom.to_string(&var_names));
    }

    #[test]
    fn poly_creation() {
        let var_names = ["q0", "_1_", "_2_", "z", "z_1", "t0", "t1", "t3", "t4"];
        let mut poly = BPolynom::from([(4, [7, 7]), (2, [6, 6]), (1, [5, 5]), (-3, [0, 4])]);
        println!("{}", poly.to_string(&var_names, " "));
        let monom = Monom::from((7, [3]));
        poly = poly * monom;
        println!("{}", poly.to_string(&var_names, " "));
    }

    #[test]
    fn poly_addition() {
        let var_names = ["q0", "q1", "q2", "q3", "t0", "t1", "t2", "t3"];
        let poly1 = BPolynom::from([(-8, vec![3]), (4, vec![2]), (2, vec![1]), (1, vec![0]), (4, vec![])]);
        let poly2 = BPolynom::from([(-8, [7]), (4, [6]), (2, [5]), (1, [4])]);

        let add_poly = poly1.clone() + &poly2;
        println!("  {}\n+ {}\n= {}", poly1.to_string(&var_names, " "), poly2.to_string(&var_names, " "), add_poly.to_string(&var_names, " "));
    }

    #[test]
    fn poly_mult() {
        let var_names = ["q0", "q1", "q2", "q3", "t0", "t1", "t2", "t3"];
        let poly1 = BPolynom::from([(-8, vec![3]), (4, vec![2]), (2, vec![1]), (1, vec![0]), (-3, vec![])]);
        let poly2 = BPolynom::from([(-8, [7]), (4, [6]), (2, [5]), (1, [4])]);

        let add_poly = &poly1 * &poly2;
        println!("  {}\n· {}\n= {}", poly1.to_string(&var_names, " "), poly2.to_string(&var_names, " "), add_poly.to_string(&var_names, " "));
    }

    #[test]
    fn test_engine() {
        let mut en = PolyEngine::new(BPolynom::empty());
        let upper = 4;
        let vars: Vec<usize> = (207..(207 + upper)).collect();
        let var_names = (0..upper).map(|i| format!("q{}", i)).collect();
        let q_poly = en.get_2_compl_poly(vars, var_names);
        println!("{}", q_poly.to_string(&en.var_names, " "));
        println!("{:?}", en.var_mapping);
        println!("{:?}", en.var_names);
        println!("{:?}", en.reverse_mapping);
        println!("{:?}", en.free_var_slots);
        en.add_from_generates(q_poly.clone());
        println!("{}", en.p.to_string(&en.var_names, " "));
        println!("{:?}", en.var_mapping);
        println!("{:?}", en.var_names);
        println!("{:?}", en.reverse_mapping);
        println!("{:?}", en.free_var_slots);
        en.not_replace(208, 211, String::from("kuhkacke"));
        println!("{}", en.p.to_string(&en.var_names, " "));
        println!("{:?}", en.var_mapping);
        println!("{:?}", en.var_names);
        println!("{:?}", en.reverse_mapping);
        println!("{:?}", en.free_var_slots);
        en.xor_replace(207, 255, String::from("pf1"), 256, String::from("pf2"));
        println!("{}", en.p.to_string(&en.var_names, " "));
        println!("{:?}", en.var_mapping);
        println!("{:?}", en.var_names);
        println!("{:?}", en.reverse_mapping);
        println!("{:?}", en.free_var_slots);
        en.or_replace(256, 260, String::from("or1"), 261, String::from("or2"));
        println!("{}", en.p.to_string(&en.var_names, "\n"));
        println!("{:?}", en.var_mapping);
        println!("{:?}", en.var_names);
        println!("{:?}", en.reverse_mapping);
        println!("{:?}", en.free_var_slots);
        en.and_replace(211, 270, String::from("dreck"), 271, String::from("vogel"));
        println!("{}", en.p.to_string(&en.var_names, "\n"));
        println!("{:?}", en.var_mapping);
        println!("{:?}", en.var_names);
        println!("{:?}", en.reverse_mapping);
        println!("{:?}", en.free_var_slots);
    }

    #[test]
    fn en_ha() {
        let mut en = PolyEngine::new(BPolynom::empty());
        let upper = 2;
        let vars: Vec<usize> = (207..(207 + upper)).collect();
        let names = (0..upper).map(|i| format!("S{}", i)).collect();
        let sum = en.get_unsigned_poly(vars, names);
        let vars: Vec<usize> = (209..(209 + 1)).collect();
        let names = (0..1).map(|i| format!("A{}", i)).collect();
        let a = en.get_unsigned_poly(vars, names);
        let vars: Vec<usize> = (210..(210 + 1)).collect();
        let names = (0..1).map(|i| format!("B{}", i)).collect();
        let b = en.get_unsigned_poly(vars, names);
        println!("{}", sum.to_string(&en.var_names, " "));
        println!("{}", a.to_string(&en.var_names, " "));
        println!("{}", b.to_string(&en.var_names, " "));
        en.add_from_generates(sum);
        println!("{}", en.p.to_string(&en.var_names, " "));
        en.and_replace(208, 209, "A0".into(), 210, "B0".into());
        println!("{}", en.p.to_string(&en.var_names, " "));
        en.xor_replace(207, 209, "A0".into(), 210, "B0".into());
        println!("{}", en.p.to_string(&en.var_names, " "));
    }

    #[test]
    fn en_fa() {
        let mut en = PolyEngine::new(BPolynom::empty());
        let upper = 2;
        let vars: Vec<usize> = (207..(207 + upper)).collect();
        let names = (0..upper).map(|i| format!("S{}", i)).collect();
        let sum = en.get_unsigned_poly(vars, names);
        let vars: Vec<usize> = (209..(209 + 1)).collect();
        let names = (0..1).map(|i| format!("A{}", i)).collect();
        let a = en.get_unsigned_poly(vars, names);
        let vars: Vec<usize> = (210..(210 + 1)).collect();
        let names = (0..1).map(|i| format!("B{}", i)).collect();
        let b = en.get_unsigned_poly(vars, names);
        println!("{}", sum.to_string(&en.var_names, " "));
        println!("{}", a.to_string(&en.var_names, " "));
        println!("{}", b.to_string(&en.var_names, " "));
        en.add_from_generates(sum);
        println!("{}", en.p.to_string(&en.var_names, " "));
        en.print_var_occurences();
        en.or_replace(208, 211, "G0".into(), 212, "G1".into());
        println!("{}", en.p.to_string(&en.var_names, " "));
        en.print_var_occurences();
        en.and_replace(212, 213, "C0".into(), 214, "G2".into());
        println!("{}", en.p.to_string(&en.var_names, " "));
        en.print_var_occurences();
        en.xor_replace(207, 213, "C0".into(), 214, "G2".into());
        println!("{}", en.p.to_string(&en.var_names, " "));
        en.print_var_occurences();
        en.and_replace(211, 209, "A0".into(), 210, "B0".into());
        println!("{}", en.p.to_string(&en.var_names, " "));
        en.print_var_occurences();
        en.xor_replace(214, 209, "A0".into(), 210, "B0".into());
        println!("{}", en.p.to_string(&en.var_names, " "));
        en.print_var_occurences();
    }

    #[test]
    fn en_fa_2() {
        let mut en = PolyEngine::new(BPolynom::empty());
        let upper = 2;
        let vars: Vec<usize> = (207..(207 + upper)).collect();
        let names = (0..upper).map(|i| format!("S{}", i)).collect();
        let sum = en.get_unsigned_poly(vars, names);
        let vars: Vec<usize> = (209..(209 + 1)).collect();
        let names = (0..1).map(|i| format!("A{}", i)).collect();
        let a = en.get_unsigned_poly(vars, names);
        let vars: Vec<usize> = (210..(210 + 1)).collect();
        let names = (0..1).map(|i| format!("B{}", i)).collect();
        let b = en.get_unsigned_poly(vars, names);
        println!("{}", sum.to_string(&en.var_names, " "));
        println!("{}", a.to_string(&en.var_names, " "));
        println!("{}", b.to_string(&en.var_names, " "));
        en.add_from_generates(sum);
        println!("{}", en.p.to_string(&en.var_names, " "));

        en.or_replace(208, 211, "G0".into(), 212, "G1".into());
        println!("{}\n", en.p.to_string(&en.var_names, " "));

        en.and_replace(212, 213, "C0".into(), 214, "G2".into());
        println!("{}\n", en.p.to_string(&en.var_names, " "));

        en.xor_replace(207, 213, "C0".into(), 214, "G2".into());
        println!("{}\n", en.p.to_string(&en.var_names, " "));

        en.and_replace(211, 209, "A0".into(), 210, "B0".into());
        println!("{}\n", en.p.to_string(&en.var_names, " "));

        en.xor_replace(214, 209, "A0".into(), 210, "B0".into());
        println!("{}\n", en.p.to_string(&en.var_names, " "));

        en.const_1_replace(213);
        println!("{}\n", en.p.to_string(&en.var_names, " "));

        en.const_1_replace(209);
        println!("{}\n", en.p.to_string(&en.var_names, " "));

        en.const_0_replace(210);
        println!("{}\n", en.p.to_string(&en.var_names, " "));
    }
}
