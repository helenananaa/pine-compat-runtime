use crate::*;

impl HistoricalRuntime<'_> {
    pub(crate) fn reject_request_object_graph(
        &self,
        value: &PineValue,
    ) -> Result<(), RuntimeError> {
        let mut pending = vec![value.clone()];
        let mut seen = std::collections::HashSet::new();
        while let Some(value) = pending.pop() {
            match value {
                PineValue::UserTypeRef(_) => {
                    return Err(RuntimeError {
                        message: "request.security UDT object graph transfer is not implemented"
                            .to_owned(),
                    });
                }
                PineValue::UserType(fields) | PineValue::Tuple(fields) => pending.extend(fields),
                PineValue::Array(id) if seen.insert((0, id)) => {
                    if let Some(values) = self.array_values_clone(id)? {
                        pending.extend(values);
                    }
                }
                PineValue::Matrix(id) if seen.insert((1, id)) => {
                    if let Some(matrix) = self.matrix_store.get(&id) {
                        pending.extend(matrix.values.clone());
                    }
                }
                PineValue::Map(id) if seen.insert((2, id)) => {
                    if let Some(map) = self.map_store.get(&id) {
                        pending.extend(
                            map.entries
                                .iter()
                                .flat_map(|(key, value)| [key.clone(), value.clone()]),
                        );
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub(crate) fn materialize_object(&self, value: &PineValue) -> Result<PineValue, RuntimeError> {
        self.materialize_object_inner(value, &mut std::collections::HashSet::new())
    }

    fn materialize_object_inner(
        &self,
        value: &PineValue,
        seen: &mut std::collections::HashSet<u32>,
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::UserTypeRef(id) = value else {
            return Ok(value.clone());
        };
        if !seen.insert(*id) {
            return Err(RuntimeError {
                message: "cyclic UDT cannot be materialized as a value tree".to_owned(),
            });
        }
        let fields = self
            .object_store
            .get(*id as usize)
            .ok_or_else(|| RuntimeError {
                message: "invalid UDT object reference".to_owned(),
            })?;
        let fields = fields
            .iter()
            .map(|field| self.materialize_object_inner(field, seen))
            .collect::<Result<_, _>>()?;
        seen.remove(id);
        Ok(PineValue::UserType(fields))
    }

    pub(crate) fn allocate_object(
        &mut self,
        fields: Vec<PineValue>,
        identity: &pine_ir::HirUserTypeIdentity,
    ) -> Result<PineValue, RuntimeError> {
        let id = u32::try_from(self.object_store.len()).map_err(|_| RuntimeError {
            message: "UDT object identity space exhausted".to_owned(),
        })?;
        let flags = self
            .program
            .user_types
            .iter()
            .find(|ty| {
                ty.identity.source_id == identity.source_id
                    && ty.declaration_name == identity.type_name
            })
            .map_or_else(
                || vec![false; fields.len()],
                |ty| ty.fields.iter().map(|field| field.varip).collect(),
            );
        self.object_store.push(fields);
        self.object_varip_fields.push(flags);
        Ok(PineValue::UserTypeRef(id))
    }

    pub(crate) fn seed_intrabar_objects_from(
        &mut self,
        previous: &Self,
        mut roots: Vec<PineValue>,
    ) {
        let committed_count = self.object_store.len();
        for id in 0..committed_count {
            if let (Some(fields), Some(flags)) = (
                previous.object_store.get(id),
                self.object_varip_fields.get(id),
            ) {
                for (index, varip) in flags.iter().enumerate() {
                    if *varip && let Some(value) = fields.get(index) {
                        self.object_store[id][index] = value.clone();
                        roots.push(value.clone());
                    }
                }
            }
        }
        let mut seen = std::collections::HashSet::new();
        while let Some(value) = roots.pop() {
            match value {
                PineValue::UserTypeRef(id) if seen.insert((0, id)) => {
                    let end = id as usize + 1;
                    if end > self.object_store.len() && end <= previous.object_store.len() {
                        // Preserve identity allocation slots. New varip-retained objects
                        // have no committed field state to roll back to.
                        let start = self.object_store.len();
                        self.object_store
                            .extend_from_slice(&previous.object_store[start..end]);
                        self.object_varip_fields
                            .extend_from_slice(&previous.object_varip_fields[start..end]);
                    }
                    if let Some(fields) = self.object_store.get(id as usize) {
                        roots.extend(fields.clone());
                    }
                }
                PineValue::UserType(fields) | PineValue::Tuple(fields) => roots.extend(fields),
                PineValue::Array(id) if seen.insert((1, id)) => {
                    if let Some(values) = self.array_store.get(&id) {
                        roots.extend(values.clone());
                    }
                }
                PineValue::Map(id) if seen.insert((2, id)) => {
                    if let Some(map) = self.map_store.get(&id) {
                        roots.extend(
                            map.entries
                                .iter()
                                .flat_map(|(key, value)| [key.clone(), value.clone()]),
                        );
                    }
                }
                PineValue::Matrix(id) if seen.insert((3, id)) => {
                    if let Some(matrix) = self.matrix_store.get(&id) {
                        roots.extend(matrix.values.clone());
                    }
                }
                _ => {}
            }
        }
    }

    pub(crate) fn object_field(&self, id: u32, index: usize) -> Result<PineValue, RuntimeError> {
        self.object_store
            .get(id as usize)
            .and_then(|fields| fields.get(index))
            .cloned()
            .ok_or_else(|| RuntimeError {
                message: "invalid UDT object reference or field".to_owned(),
            })
    }

    pub(crate) fn set_object_field(
        &mut self,
        id: u32,
        index: usize,
        value: PineValue,
    ) -> Result<(), RuntimeError> {
        let field = self
            .object_store
            .get_mut(id as usize)
            .and_then(|fields| fields.get_mut(index))
            .ok_or_else(|| RuntimeError {
                message: "invalid UDT object reference or field".to_owned(),
            })?;
        *field = value;
        Ok(())
    }
}
