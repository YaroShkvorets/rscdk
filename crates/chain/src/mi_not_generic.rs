use crate::db::{
    Idx64Table,
    Idx128Table,
    Idx256Table,
    // IdxF128Table,
    SecondaryType,
    SecondaryValue,
    SecondaryIterator,
    IdxTable,
    MultiIndexValue,
};

use crate::vmapi::db::*;
use crate::name::{
    Name,
};

use crate::{
    vec::Vec,
};

use crate::{
    check,
};

use crate::boxed::Box;
use crate::serializer::{
    Encoder,
};

pub fn cast_value<T: MultiIndexValue>(value: &Option<Box<dyn MultiIndexValue>>) -> Option<&T> {
    if let Some(x) = value {
        x.as_any().downcast_ref::<T>()
    } else {
        None
    }
}

///
pub struct Iterator<'a> {
    ///
    pub i: i32,
    primary: Option<u64>,
    db: &'a TableI64,
}

impl<'a> Iterator<'a> {
    ///
    pub fn new(i: i32, primary: Option<u64>, db: &'a TableI64) -> Self {
        Self { i, primary, db }
    }

    ///
    pub fn get_primary(&self) -> Option<u64> {
        if !self.is_ok() {
            return None;
        }

        if self.primary.is_some() {
            return self.primary;
        }
        
        let value = self.db.get(self).unwrap();
        return Some(value.get_primary());
    }

    pub fn get_value(&self) -> Option<Box<dyn MultiIndexValue>> {
        return self.db.get(self);
    }

    pub fn get_value_ex<T: MultiIndexValue + core::clone::Clone>(&self) -> Option<T> {
        let value = self.get_value();
        if let Some(x) = cast_value::<T>(&value) {
            Some(x.clone())
        } else {
            None
        }
    }

    ///
    pub fn is_ok(&self) -> bool {
        self.i >= 0
    }

    ///
    pub fn is_end(&self) -> bool {
        return self.i < -1;
    }

    ///
    pub fn expect(self, msg: &str) -> Self {
        check(self.is_ok(), msg);            
        return self;
    }

    ///
    pub fn expect_not_ok(self, msg: &str) -> Self {
        check(!self.is_ok(), msg);            
        return self;
    }
}


///
pub struct TableI64 {
    ///
    pub code: u64,
    ///
    pub scope: u64,
    ///
    pub table: u64,
    unpacker: fn(&[u8]) -> Box<dyn MultiIndexValue>,
}

impl TableI64 {
    ///
    pub fn new(code: Name, scope: Name, table: Name, unpacker: fn(&[u8]) -> Box<dyn MultiIndexValue>) -> Self {
        TableI64 {
            code: code.value(),
            scope: scope.value(),
            table: table.value(),
            unpacker,
        }
    }

    ///
    pub fn store(&self, id: u64,  data: &[u8], payer: Name) -> Iterator {
        let it = db_store_i64(self.scope, self.table, payer.value(), id, data.as_ptr(), data.len() as u32);
        Iterator { i: it, primary: Some(id), db: self }
    }

    ///
    pub fn update(&self, iterator: &Iterator, value: &dyn MultiIndexValue, payer: Name) {
        let mut enc = Encoder::new(value.size());
        value.pack(&mut enc);
        let data = enc.get_bytes();
        db_update_i64(iterator.i, payer.value(), data.as_ptr(), data.len() as u32);
    }

    ///
    pub fn remove(&self, iterator: &Iterator) {
        db_remove_i64(iterator.i);
    }

    ///
    pub fn get(&self, iterator: &Iterator) -> Option<Box<dyn MultiIndexValue>> {
        if !iterator.is_ok() {
            return None;
        }

        let data = db_get_i64(iterator.i);
        return Some((self.unpacker)(&data));
    }

    ///
    pub fn next(&self, iterator: &Iterator) -> Iterator {
        let mut primary = 0;
        let it = db_next_i64(iterator.i, &mut primary);
        if it >= 0 {
            Iterator { i: it, primary: Some(primary), db: self }
        } else {
            Iterator { i: it, primary: None, db: self }
        }
    }

    ///
    pub fn previous(&self, iterator: &Iterator) -> Iterator {
        let mut primary = 0;
        let it = db_previous_i64(iterator.i, &mut primary);
        if it >= 0 {
            Iterator { i: it, primary: Some(primary), db: self }
        } else {
            Iterator { i: it, primary: None, db: self }
        }
    }

    ///
    pub fn find(&self, primary_key: u64) -> Iterator {
        let it = db_find_i64(self.code, self.scope, self.table, primary_key);
        Iterator { i: it, primary: Some(primary_key), db: self }
    }

    ///
    pub fn lower_bound(&self, id: u64) -> Iterator {
        let it = db_lowerbound_i64(self.code, self.scope, self.table, id);
        Iterator { i: it, primary: None, db: self }
    }

    ///
    pub fn upper_bound(&self, id: u64) -> Iterator {
        let it = db_upperbound_i64(self.code, self.scope, self.table, id);
        Iterator { i: it, primary: None, db: self }
    }

    ///
    pub fn end(&self) -> Iterator {
        let it = db_end_i64(self.code, self.scope, self.table);
        Iterator { i: it, primary: None, db: self }
    }
}

///
pub struct MultiIndex {
    ///
    pub code: Name,
    ///
    pub scope: Name,
    ///
    pub table: Name,
    ///
    pub db: TableI64,
    ///
    pub idxdbs: Vec<Box<dyn IdxTable>>,
    ///
    pub unpacker: fn(&[u8]) -> Box<dyn MultiIndexValue>,
}

impl MultiIndex {
    ///
    pub fn new(code: Name, scope: Name, table: Name, indices: &[SecondaryType], unpacker: fn(&[u8]) -> Box<dyn MultiIndexValue>) -> Self {
        let mut idxdbs: Vec<Box<dyn IdxTable>> = Vec::new();
        let mut i: usize = 0;
        let idx_table = table.value() & 0xfffffffffffffff0;
        for idx in indices {
            match idx {
                SecondaryType::Idx64 => idxdbs.push(
                    Box::new(Idx64Table::new(i, code, scope, Name::from_u64(idx_table + i as u64)))
                ),
                SecondaryType::Idx128 => idxdbs.push(
                    Box::new(Idx128Table::new(i, code, scope, Name::from_u64(idx_table + i as u64)))
                ),
                SecondaryType::Idx256 => idxdbs.push(
                    Box::new(Idx256Table::new(i, code, scope, Name::from_u64(idx_table + i as u64)))
                ),
                _ => panic!("unsupported secondary index type"),
            }
            i += 1;
        }
        MultiIndex {
            code,
            scope,
            table,
            db: TableI64::new(code, scope, table, unpacker),
            idxdbs,
            unpacker: unpacker,
        }
    }

    ///
    pub fn store(&self, value: &dyn MultiIndexValue, payer: Name) -> Iterator {
        let primary = value.get_primary();
        for i in 0..self.idxdbs.len() {
            let v2 = value.get_secondary_value(i);
            self.idxdbs[i].store(primary, v2, payer);
        }
        let mut enc = Encoder::new(value.size());
        value.pack(&mut enc);
        let it = self.db.store(primary, enc.get_bytes(), payer);
        return it;
    }

    ///
    pub fn update(&self, iterator: &Iterator, value: &dyn MultiIndexValue, payer: Name) {
        check(iterator.is_ok(), "update: invalid iterator");
        let primary = iterator.get_primary().unwrap();
        for i in 0..self.idxdbs.len() {
            let v2 = value.get_secondary_value(i);
            let (it_secondary, secondary_value) = self.idxdbs[i].find_primary(primary);
            if secondary_value == v2 {
                continue;
            }
            self.idxdbs[i].update(&it_secondary, v2, payer);
        }
        self.db.update(iterator, value, payer);
    }

    pub fn set(&self, value: &dyn MultiIndexValue, payer: Name) {
        let primary = value.get_primary();
        let it = self.find(primary);
        if it.is_ok() {
            self.update(&it, value, payer);
        } else {
            self.store(value, payer);
        }
    }

    ///
    pub fn remove(&self, iterator: &Iterator) {
        if !iterator.is_ok() {
            return;
        }
        let primary = iterator.get_primary().unwrap();

        for i in 0..self.idxdbs.len() {
            let (it_secondary, _) = self.idxdbs[i].find_primary(primary);
            self.idxdbs[i].remove(&it_secondary);
        }
        self.db.remove(iterator);
    }

    ///
    pub fn get(&self, iterator: &Iterator) -> Option<Box<dyn MultiIndexValue>> {
        if !iterator.is_ok() {
            return None;
        }

        return self.db.get(iterator);
    }

    ///
    pub fn get_by_primary(&self, primary: u64) -> Option<Box<dyn MultiIndexValue>> {
        let it = self.db.find(primary);
        return self.get(&it);
    }

    ///
    pub fn next(&self, iterator: &Iterator) -> Iterator {
        return self.db.next(iterator);
    }

    ///
    pub fn previous(&self, iterator: &Iterator) -> Iterator {
        return self.db.previous(iterator);
    }

    ///
    pub fn find(&self, id: u64) -> Iterator {
        return self.db.find(id);
    }

    ///
    pub fn lower_bound(&self, id: u64) -> Iterator {
        return self.db.lower_bound(id);
    }

    ///
    pub fn upper_bound(&self, id: u64) -> Iterator {
        return self.db.upper_bound(id);
    }

    ///
    pub fn end(&self) -> Iterator {
        return self.db.end();
    }

    ///
    pub fn get_idx_db(&self, i: usize) -> &dyn IdxTable {
        return self.idxdbs[i].as_ref();
    }

    ///
    pub fn idx_update(&self, it: &SecondaryIterator, value: SecondaryValue, payer: Name) {
        check(it.is_ok(), "idx_update: invalid iterator");

        let it_primary = self.find(it.primary);
        let mut db_value = self.get(&it_primary).unwrap();
        let idx_db = self.idxdbs[it.db_index].as_ref();
        db_value.set_secondary_value(idx_db.get_db_index(), value);
        self.update(&it_primary, db_value.as_ref(), payer);
        idx_db.update(it, value, payer);    
    }
}
