use crate::utils::non_nil_uuid::NonNilUuid;
use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::mem::MaybeUninit;
use std::rc::{Rc, Weak};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "req_state")]
enum ReqData<T> {
    Ready(T),
    Pending,
    Taken,
}

impl<T: 'static> ReqData<T> {
    fn is_pending(&self) -> bool {
        match self {
            ReqData::Ready(_) => false,
            ReqData::Pending => true,
            ReqData::Taken => false,
        }
    }

    fn to_option(self) -> Option<T> {
        match self {
            ReqData::Ready(v) => Some(v),
            ReqData::Pending => None,
            ReqData::Taken => todo!(),
        }
    }

    fn to_any(self) -> AnyReqData {
        match self {
            ReqData::Ready(data) => ReqData::Ready(Box::new(data)),
            ReqData::Pending => ReqData::Pending,
            ReqData::Taken => ReqData::Taken,
        }
    }
}

impl<T> ReqData<&mut T> {
    fn take(&mut self) -> Result<T, ReqTakeError> {
        match self {
            ReqData::Ready(v) => {}
            ReqData::Pending => Err(ReqTakeError::Pending)?,
            ReqData::Taken => Err(ReqTakeError::AlreadyTaken)?,
        }

        unsafe {
            match std::mem::replace(self, ReqData::Taken) {
                ReqData::Ready(v) => Ok(std::mem::replace(v, MaybeUninit::uninit().assume_init())),
                _ => unreachable!(),
            }
        }
    }
}

type AnyReqData = ReqData<Box<dyn Any>>;

impl AnyReqData {
    fn downcast_ref<T: 'static>(&self) -> Option<ReqData<&T>> {
        match self {
            ReqData::Ready(data) => data.downcast_ref().map(ReqData::Ready),
            ReqData::Pending => Some(ReqData::Pending),
            ReqData::Taken => Some(ReqData::Taken),
        }
    }

    fn downcast_mut<T: 'static>(&mut self) -> Option<ReqData<&mut T>> {
        match self {
            ReqData::Ready(data) => data.downcast_mut().map(ReqData::Ready),
            ReqData::Pending => Some(ReqData::Pending),
            ReqData::Taken => Some(ReqData::Taken),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReqPromise<T> {
    #[serde(skip)]
    data: Option<Weak<RefCell<AnyReqData>>>,
    id: NonNilUuid,
    #[serde(skip)]
    _pd: std::marker::PhantomData<T>,
}

impl<T> ReqPromise<T> {
    pub fn new() -> (Self, ReqFuture<T>) {
        let data = Rc::new(RefCell::new(ReqData::Pending));
        let id = NonNilUuid::new_v4();
        (
            Self {
                data: Some(Rc::downgrade(&data)),
                id,
                _pd: Default::default(),
            },
            ReqFuture {
                data,
                id,
                _pd: Default::default(),
            },
        )
    }

    pub fn make_ready(&mut self, context: &ReqContext, value: T) -> Result<(), ReqMakeReadyError>
    where
        T: 'static,
    {
        match &self.data {
            None => {
                let data = context.data.borrow();

                let weak = data.get(&self.id).ok_or(ReqMakeReadyError::Expired)?;

                self.data = Some(weak.clone());
                match weak.upgrade() {
                    None => {
                        context.data.borrow_mut().remove(&self.id);
                        Err(ReqMakeReadyError::Expired)
                    }
                    Some(rc) => {
                        let mut data = rc.borrow_mut();
                        if data.is_pending() {
                            Ok(*data = ReqData::Ready(Box::new(value)))
                        } else {
                            Err(ReqMakeReadyError::AlreadyMadeReady)
                        }
                    }
                }
            }
            Some(data) => match data.upgrade() {
                None => Err(ReqMakeReadyError::Expired),
                Some(rc) => {
                    let mut data = rc.borrow_mut();
                    if data.is_pending() {
                        Ok(*data = ReqData::Ready(Box::new(value)))
                    } else {
                        Err(ReqMakeReadyError::AlreadyMadeReady)
                    }
                }
            },
        }
    }
}

#[derive(Debug)]
pub struct ReqFuture<T> {
    data: Rc<RefCell<AnyReqData>>,
    id: NonNilUuid,
    _pd: std::marker::PhantomData<T>,
}

pub struct ReqFutureSeed<'context, T> {
    context: &'context ReqContext,
    _pd: std::marker::PhantomData<T>,
}

impl<'context, T> Clone for ReqFutureSeed<'context, T> {
    fn clone(&self) -> Self {
        Self {
            context: self.context,
            _pd: Default::default(),
        }
    }
}

impl<'context, T> ReqFutureSeed<'context, T> {
    pub fn new(context: &'context ReqContext) -> Self {
        Self {
            context,
            _pd: Default::default(),
        }
    }
}

impl<T: Serialize + 'static> Serialize for ReqFuture<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Impl<'a, T> {
            data: ReqData<&'a T>,
            id: NonNilUuid,
        }

        Impl {
            data: self.data.borrow().downcast_ref::<T>().unwrap(),
            id: self.id,
        }
        .serialize(serializer)
    }
}

impl<'de, 'context, T: Deserialize<'de> + 'static> DeserializeSeed<'de>
    for ReqFutureSeed<'context, T>
{
    type Value = ReqFuture<T>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Impl<T> {
            data: ReqData<T>,
            id: NonNilUuid,
        }

        let Impl { data, id } = Impl::deserialize(deserializer)?;
        Ok(self.context.register(data, id))
    }
}

impl<T> ReqFuture<T> {
    // pub fn value(&mut self) -> Option<&T> {
    //     self.data.borrow().downcast_ref().unwrap().to_option()
    // }

    pub fn take(&mut self) -> Result<T, ReqTakeError>
    where
        T: 'static,
    {
        let mut data = self.data.borrow_mut();
        data.downcast_mut().unwrap().take()
    }
}

#[derive(Debug)]
pub enum ReqMakeReadyError {
    Expired,
    AlreadyMadeReady,
}

impl Display for ReqMakeReadyError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ReqMakeReadyError {}

#[derive(Debug)]
pub enum ReqTakeError {
    Pending,
    AlreadyTaken,
}

impl Display for ReqTakeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ReqTakeError {}

pub struct ReqContext {
    data: RefCell<BTreeMap<NonNilUuid, Weak<RefCell<ReqData<Box<dyn Any>>>>>>,
}

impl ReqContext {
    pub fn new() -> Self {
        Self {
            data: RefCell::new(BTreeMap::new()),
        }
    }

    fn register<T: 'static>(&self, data: ReqData<T>, id: NonNilUuid) -> ReqFuture<T> {
        let data = Rc::new(RefCell::new(data.to_any()));
        self.data
            .borrow_mut()
            .try_insert(id, Rc::downgrade(&data))
            .unwrap();
        ReqFuture {
            data,
            id,
            _pd: Default::default(),
        }
    }
}
