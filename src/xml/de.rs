use super::error::*;
use super::error::ErrorCode::*;
use de;

use std::str::from_utf8;


pub struct Deserializer<Iter: Iterator<Item=u8>> {
    rdr: Iter,
    ch: Option<u8>,
    line: usize,
    col: usize,
    buf: Vec<u8>,
}

pub struct InnerDeserializer<'a, Iter: Iterator<Item=u8> + 'a> (
    &'a mut Deserializer<Iter>,
);

impl<'a, Iter> de::Deserializer for InnerDeserializer<'a, Iter>
where Iter: Iterator<Item=u8>,
{
    type Error = Error;

    #[inline]
    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        println!("InnerDeserializer::visit");
        self.0.buf.clear();
        match self.0.ch.unwrap() {
            b'/' => {
                self.0.bump();
                assert!(self.0.ch_is(b'>'));
                self.0.bump();
                self.0.skip_whitespace();
                assert!(self.0.ch_is(b'<'));
                self.0.bump();
                try!(self.0.read_tag());
                visitor.visit_unit()
            }
            b'>' => {
                self.0.bump();
                try!(self.0.read_until(b'<'));
                println!("{:?}", from_utf8(&self.0.buf));
                let v = visitor.visit_str(try!(KeyDeserializer::from_utf8(self.0)));
                self.0.buf.clear();
                self.0.bump();
                assert!(self.0.ch_is(b'/'));
                self.0.bump();
                try!(self.0.skip_until(b'>'));
                self.0.bump();
                self.0.skip_whitespace();
                assert!(self.0.ch_is(b'<'));
                self.0.bump();
                try!(self.0.read_tag());
                v
            }
            _ => Err(self.0.error(RawValueCannotHaveAttributes))
        }
    }

    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        self.0.buf.clear();
        match self.0.ch.unwrap() {
            b'/' => {
                self.0.bump();
                assert!(self.0.ch_is(b'>'));
                self.0.bump();
                visitor.visit_none()
            }
            b'>' | b' ' | b'\n' | b'\r' | b'\t' => {
                visitor.visit_some(self)
            }
            _ => Err(self.0.error(InvalidOptionalElement))
        }
    }

    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_seq(SeqVisitor::new(self.0))
    }

    fn visit_map<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        self.0.buf.clear();
        println!("{:?} __ {:?}", self.0.buf, self.0.ch);
        let v = try!(self.0.parse_inner_map(visitor));
        self.0.skip_whitespace();
        assert!(self.0.ch_is(b'<'));
        self.0.bump();
        try!(self.0.read_tag());
        Ok(v)
    }

    fn visit_named_unit<V>(&mut self, _name: &str, _visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        unimplemented!()
    }

    fn visit_named_seq<V>(&mut self, _name: &str, _visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        unimplemented!()
    }
}

pub struct KeyDeserializer<'a> (
    &'a str,
);

impl<'a> KeyDeserializer<'a> {
    fn decode<T, Iter>(de: &Deserializer<Iter>) -> Result<T, Error>
        where Iter: Iterator<Item=u8>,
        T: de::Deserialize,
    {
        let s = from_utf8(&de.buf);
        match s {
            Ok(text) => {
                let kds = &mut KeyDeserializer(text);
                let val = de::Deserialize::deserialize(kds);
                if val.is_err() {
                    println!("decode, err");
                }
                Ok(try!(val))
            },
            Err(_) => Err(de.error(NotUtf8))
        }
    }

    fn from_utf8<Iter>(de: &Deserializer<Iter>) -> Result<&str, Error>
        where Iter: Iterator<Item=u8>,
    {
        let s = from_utf8(&de.buf);
        s.or(Err(de.error(NotUtf8)))
    }
}

impl<'a> de::Deserializer for KeyDeserializer<'a> {
    type Error = Error;

    #[inline]
    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        println!("{:?} keydeserializer::visit", self as *const Self);
        println!("{:?} {:?}", self as *const Self, self.0);
        match visitor.visit_str(self.0) {
            Ok(x) => Ok(x),
            Err(x) => {println!("err"); Err(x)},
        }
    }

    #[inline]
    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        println!("{:?} keydeserializer::visit_option", self as *const Self);
        if self.0.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    #[inline]
    fn visit_enum<V>(&mut self, _enum: &str, _visitor: V) -> Result<V::Value, Error>
        where V: de::EnumVisitor,
    {
        unimplemented!()
    }

    #[inline]
    fn visit_seq<V>(&mut self, _visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        unimplemented!()
    }
}

impl<Iter> Deserializer<Iter>
    where Iter: Iterator<Item=u8>,
{
    /// Creates the Xml parser.
    #[inline]
    pub fn new(rdr: Iter) -> Deserializer<Iter> {
        let mut p = Deserializer {
            rdr: rdr,
            ch: Some(b'\x00'),
            line: 1,
            col: 0,
            buf: Vec::with_capacity(128),
        };
        p.bump();
        return p;
    }

    #[inline]
    pub fn end(&mut self) -> Result<(), Error> {
        self.skip_whitespace();
        assert!(self.eof());
        assert!(self.buf.is_empty());
        Ok(())
    }

    fn eof(&self) -> bool { self.ch.is_none() }

    fn bump(&mut self) {
        print!("bump: {:?}", (self.line, self.col, self.ch));
        if let None = self.ch {
            panic!("iterator overrun");
        }
        self.ch = self.rdr.next();

        if self.ch_is(b'\n') {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        println!(" -> {:?}", self.ch);
    }

    fn ch_is(&self, c: u8) -> bool {
        self.ch == Some(c)
    }

    fn ch_is_one_of(&self, c: &[u8]) -> bool {
        for &c in c {
            if Some(c) == self.ch {
                return true;
            }
        }
        return false;
    }

    fn error(&self, reason: ErrorCode) -> Error {
        Error::SyntaxError(reason, self.line, self.col)
    }

    fn skip_whitespace(&mut self) {
        while self.ch_is_one_of(" \n\t\r".as_bytes()) { self.bump(); }
    }

    fn skip_until(&mut self, ch: u8) -> Result<(), Error> {
        while let Some(c) = self.ch {
            if ch == c {
                return Ok(())
            }
            self.bump();
        }
        Err(self.error(EOF))
    }

    fn read_until(&mut self, ch: u8) -> Result<(), Error> {
        while let Some(c) = self.ch {
            if ch == c {
                return Ok(())
            }
            self.buf.push(c);
            self.bump();
        }
        Err(self.error(EOF))
    }

    fn read_tag(&mut self) -> Result<(), Error> {
        println!("read_tag");
        while let Some(c) = self.ch {
            if self.ch_is_one_of(b" \n\t\r>/") {
                return Ok(());
            }
            self.buf.push(c);
            self.bump();
        }
        Err(self.error(EOF))
    }

    fn parse_inner_map<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        self.skip_whitespace();
        match self.ch {
            None => Err(self.error(EOF)),
            Some(b'/') => {
                self.bump();
                assert!(self.ch_is(b'>'));
                self.bump();
                de::Deserialize::deserialize(&mut UnitDeserializer)
            },
            Some(b'>') => {
                self.bump();
                self.skip_whitespace();
                assert!(self.ch_is(b'<'));
                self.bump();
                if self.ch_is(b'/') {
                    try!(self.skip_until(b'>'));
                    self.bump();
                    de::Deserialize::deserialize(&mut UnitDeserializer)
                } else {
                    try!(self.read_tag());
                    let val = visitor.visit_map(ContentVisitor::new_inner(self));
                    self.buf.clear();
                    val
                }
            }
            _ => unimplemented!()
        }
    }
}


impl<Iter> de::Deserializer for Deserializer<Iter>
    where Iter: Iterator<Item=u8>,
{
    type Error = Error;

    #[inline]
    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        self.skip_whitespace();
        assert!(self.ch_is(b'<'));
        try!(self.skip_until(b'>'));
        self.bump();
        try!(self.read_until(b'<'));
        self.bump();
        assert!(self.ch_is(b'/'));
        try!(self.skip_until(b'>'));
        self.bump();
        let v = visitor.visit_str(try!(KeyDeserializer::from_utf8(self)));
        self.buf.clear();
        v
    }

    #[inline]
    fn visit_option<V>(&mut self, _visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        unimplemented!()
    }

    #[inline]
    fn visit_enum<V>(&mut self, _enum: &str, _visitor: V) -> Result<V::Value, Error>
        where V: de::EnumVisitor,
    {
        unimplemented!()
    }

    #[inline]
    fn visit_seq<V>(&mut self, _visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        unimplemented!()
    }

    #[inline]
    fn visit_map<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        self.skip_whitespace();
        assert!(self.ch_is(b'<'));
        self.bump();
        try!(self.read_tag());
        self.buf.clear();
        self.parse_inner_map(visitor)
    }
}

struct UnitDeserializer;

impl de::Deserializer for UnitDeserializer {
    type Error = Error;

    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_unit()
    }

    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_none()
    }

    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_seq(EmptySeqVisitor)
    }

    fn visit_map<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_map(EmptyMapVisitor)
    }
}

struct EmptySeqVisitor;
impl de::SeqVisitor for EmptySeqVisitor {
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize,
    {
        Ok(None)
    }

    fn end(&mut self) -> Result<(), Error> { Ok(()) }
}

struct EmptyMapVisitor;
impl de::MapVisitor for EmptyMapVisitor {
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>, Error>
        where K: de::Deserialize,
    { Ok(None) }

    fn visit_value<V>(&mut self) -> Result<V, Error>
        where V: de::Deserialize,
    { unreachable!() }

    fn end(&mut self) -> Result<(), Error> { Ok(()) }

    fn missing_field<V>(&mut self, _field: &'static str) -> Result<V, Error>
        where V: de::Deserialize,
    {
        Ok(try!(de::Deserialize::deserialize(&mut UnitDeserializer)))
    }
}

struct ContentVisitor<'a, Iter: 'a>
    where Iter: Iterator<Item=u8>,
{
    de: &'a mut Deserializer<Iter>,
    attributes_done: bool,
}

impl <'a, Iter> ContentVisitor<'a, Iter>
    where Iter: Iterator<Item=u8>,
{
    fn new_inner(de: &'a mut Deserializer<Iter>) -> Self {
        ContentVisitor {
            de: de,
            attributes_done: true,
        }
    }

    fn parse_element_close(&mut self) -> Result<(), Error> {
        assert!(self.de.ch_is(b'/'));
        self.de.bump();
        try!(self.de.read_until(b'>'));
        self.de.bump();
        Ok(())
    }
}

impl<'a, Iter> de::MapVisitor for ContentVisitor<'a, Iter>
    where Iter: Iterator<Item=u8>
{
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>, Error>
        where K: de::Deserialize,
    {
        println!("{:?} visit_key: {:?}", self as *const Self, (self.attributes_done, self.de.line, self.de.col));
        if self.de.buf.is_empty() {
            return Ok(None);
        }
        let val = try!(KeyDeserializer::decode(self.de));
        if self.attributes_done {
            Ok(val)
        } else {
            unimplemented!()
        }
    }

    fn visit_value<V>(&mut self) -> Result<V, Error>
        where V: de::Deserialize,
    {
        println!("{:?} visit_value: {:?}", self as *const Self, (self.attributes_done, self.de.line, self.de.col));
        if self.attributes_done {
            let ids = &mut InnerDeserializer(self.de);
            de::Deserialize::deserialize(ids)
        } else {
            unimplemented!()
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        println!("{:?} end: {:?}", self as *const Self, (self.attributes_done, self.de.line, self.de.col));
        if self.attributes_done {
            self.parse_element_close()
        } else {
            unimplemented!()
        }
    }

    fn missing_field<V>(&mut self, field: &'static str) -> Result<V, Error>
        where V: de::Deserialize,
    {
        println!("missing field: {}", field);
        // See if the type can deserialize from a unit.
        de::Deserialize::deserialize(&mut UnitDeserializer)
    }
}

struct SeqVisitor<'a, Iter: 'a + Iterator<Item=u8>> {
    de: &'a mut Deserializer<Iter>,
    done: bool,
}

impl<'a, Iter> SeqVisitor<'a, Iter>
    where Iter: Iterator<Item=u8>,
{
    fn new(de: &'a mut Deserializer<Iter>) -> Self {
        SeqVisitor {
            de: de,
            done: false,
        }
    }
}

impl<'a, Iter> de::SeqVisitor for SeqVisitor<'a, Iter>
    where Iter: Iterator<Item=u8>,
{
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize,
    {
        println!("SeqVisitor::visit: {:?}", (self.done, self.de.ch));
        if self.done {
            return Ok(None);
        }
        // need to copy here
        // could compare closing tag with next opening tag instead
        // but that requires modification of InnerDeserializer
        let name = self.de.buf.clone();
        self.de.buf.clear();
        let val = {
            let ids = &mut InnerDeserializer(self.de);
            try!(de::Deserialize::deserialize(ids))
        };
        println!("{:?} got seq valu", self as *const Self);
        if self.de.buf.is_empty() {
            // last of the sequence and last of the map
            self.done = true;
        } else {
            // compare next element name to current
            assert!(!self.de.buf.is_empty());
            if self.de.buf != name {
                self.done = true;
            }
        }
        Ok(Some(val))
    }

    fn end(&mut self) -> Result<(), Error> {
        println!("SeqVisitor::end: {:?}", self.de.buf);
        Ok(())
    }
}

/// Decodes an xml value from an `Iterator<u8>`.
pub fn from_iter<I, T>(iter: I) -> Result<T, Error>
    where I: Iterator<Item=u8>,
          T: de::Deserialize
{
    let mut de = Deserializer::new(iter);
    let value = try!(de::Deserialize::deserialize(&mut de));

    // Make sure the whole stream has been consumed.
    try!(de.end());
    Ok(value)
}

/// Decodes an xml value from a string
pub fn from_str<'a, T>(s: &'a str) -> Result<T, Error>
    where T: de::Deserialize
{
    from_iter(s.bytes())
}
