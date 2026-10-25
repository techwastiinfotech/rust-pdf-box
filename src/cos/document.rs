use crate::cos::object::{COSDictionary, COSObject, ObjectId};
use crate::error::{PdfError, PdfResult};
use std::collections::{BTreeMap, BTreeSet};

/// Low-level PDF document holding all COS objects and the trailer dictionary.
#[derive(Debug, Clone)]
pub struct COSDocument {
    pub objects: BTreeMap<ObjectId, COSObject>,
    pub trailer: COSDictionary,
    pub version: String,
    pub max_object_number: u32,
    pub dirty_objects: BTreeSet<ObjectId>,
}

impl Default for COSDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl COSDocument {
    pub fn new() -> Self {
        Self {
            objects: BTreeMap::new(),
            trailer: COSDictionary::new(),
            version: "1.7".to_string(),
            max_object_number: 0,
            dirty_objects: BTreeSet::new(),
        }
    }

    /// Allocates a new, unique object id with generation 0.
    pub fn allocate_id(&mut self) -> ObjectId {
        self.max_object_number += 1;
        let id = ObjectId::new(self.max_object_number, 0);
        self.dirty_objects.insert(id);
        id
    }

    /// Stores a new object and returns its allocated ObjectId.
    pub fn add_object<T: Into<COSObject>>(&mut self, obj: T) -> ObjectId {
        let id = self.allocate_id();
        self.objects.insert(id, obj.into());
        id
    }

    /// Sets or replaces an object with a specific ObjectId and marks it dirty.
    pub fn set_object<T: Into<COSObject>>(&mut self, id: ObjectId, obj: T) {
        if id.num > self.max_object_number {
            self.max_object_number = id.num;
        }
        self.objects.insert(id, obj.into());
        self.dirty_objects.insert(id);
    }

    /// Retrieves an object by its ObjectId.
    pub fn get_object(&self, id: ObjectId) -> Option<&COSObject> {
        self.objects.get(&id)
    }

    /// Retrieves a mutable reference to an object by its ObjectId and marks it dirty.
    pub fn get_object_mut(&mut self, id: ObjectId) -> Option<&mut COSObject> {
        self.dirty_objects.insert(id);
        self.objects.get_mut(&id)
    }

    /// Resolves an object: if it's a Reference, fetches the target object; otherwise returns the object.
    pub fn resolve<'a>(&'a self, obj: &'a COSObject) -> &'a COSObject {
        match obj {
            COSObject::Reference(id) => self.objects.get(id).unwrap_or(&COSObject::Null),
            other => other,
        }
    }

    /// Resolves a reference or returns error if not found.
    pub fn get_resolved(&self, id: ObjectId) -> PdfResult<&COSObject> {
        self.objects
            .get(&id)
            .ok_or(PdfError::InvalidReference(id))
    }

    /// Gets the Document Catalog dictionary (/Root).
    pub fn get_catalog(&self) -> Option<&COSDictionary> {
        let root = self.trailer.get("Root")?;
        let resolved = self.resolve(root);
        resolved.as_dict()
    }

    /// Gets mutable Document Catalog dictionary.
    pub fn get_catalog_mut(&mut self) -> Option<&mut COSDictionary> {
        let root_id = self.trailer.get_reference("Root")?;
        self.dirty_objects.insert(root_id);
        self.objects.get_mut(&root_id)?.as_dict_mut()
    }

    /// Gets Document Information dictionary (/Info).
    pub fn get_info(&self) -> Option<&COSDictionary> {
        let info = self.trailer.get("Info")?;
        let resolved = self.resolve(info);
        resolved.as_dict()
    }

    /// Clears the dirty tracking set (e.g. after saving).
    pub fn clear_dirty(&mut self) {
        self.dirty_objects.clear();
    }
}
