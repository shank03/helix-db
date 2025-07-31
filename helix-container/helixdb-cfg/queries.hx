QUERY createPatient (name: String, age: I32, gender: String, blood_type: String, medical_condition: String, date_of_admission: String, discharge_date: String, insurance_provider: String, billing_amount: F64, room_number: String, admission_type: String, medication: String, test_results: String) =>
    patient <- AddN<Patient>({name: name, age: age, gender: gender, blood_type: blood_type, medical_condition: medical_condition, date_of_admission: date_of_admission, discharge_date: discharge_date, insurance_provider: insurance_provider, billing_amount: billing_amount, room_number: room_number, admission_type: admission_type, medication: medication, test_results: test_results})
    RETURN patient

QUERY getPatient (patient_id: ID) =>
    patient <- N<Patient>(patient_id)
    RETURN patient

QUERY getAllPatients () =>
    patients <- N<Patient>
    RETURN patients

QUERY updatePatient (patient_id: ID, name: String, age: I32, gender: String, blood_type: String, medical_condition: String, date_of_admission: String, discharge_date: String, insurance_provider: String, billing_amount: F64, room_number: String, admission_type: String, medication: String, test_results: String) =>
    patient <- N<Patient>(patient_id)::UPDATE({name: name, age: age, gender: gender, blood_type: blood_type, medical_condition: medical_condition, date_of_admission: date_of_admission, discharge_date: discharge_date, insurance_provider: insurance_provider, billing_amount: billing_amount, room_number: room_number, admission_type: admission_type, medication: medication, test_results: test_results})
    RETURN patient

QUERY deletePatient (patient_id: ID) =>
    DROP N<Patient>(patient_id)
    RETURN "success"

QUERY createDoctor (name: String) =>
    doctor <- AddN<Doctor>({name: name})
    RETURN doctor

QUERY getDoctor (doctor_id: ID) =>
    doctor <- N<Doctor>(doctor_id)
    RETURN doctor

QUERY getAllDoctors () =>
    doctors <- N<Doctor>
    RETURN doctors

QUERY updateDoctor (doctor_id: ID, name: String) =>
    doctor <- N<Doctor>(doctor_id)::UPDATE({name: name})
    RETURN doctor

QUERY deleteDoctor (doctor_id: ID) =>
    DROP N<Doctor>(doctor_id)
    RETURN "success"

QUERY createHospital (name: String) =>
    hospital <- AddN<Hospital>({name: name})
    RETURN hospital

QUERY getHospital (hospital_id: ID) =>
    hospital <- N<Hospital>(hospital_id)
    RETURN hospital

QUERY getAllHospitals () =>
    hospitals <- N<Hospital>
    RETURN hospitals

QUERY updateHospital (hospital_id: ID, name: String) =>
    hospital <- N<Hospital>(hospital_id)::UPDATE({name: name})
    RETURN hospital

QUERY deleteHospital (hospital_id: ID) =>
    DROP N<Hospital>(hospital_id)
    RETURN "success"

QUERY createDepartment (name: String) =>
    department <- AddN<Department>({name: name})
    RETURN department

QUERY getDepartment (department_id: ID) =>
    department <- N<Department>(department_id)
    RETURN department

QUERY getAllDepartments () =>
    departments <- N<Department>
    RETURN departments

QUERY updateDepartment (department_id: ID, name: String) =>
    department <- N<Department>(department_id)::UPDATE({name: name})
    RETURN department

QUERY deleteDepartment (department_id: ID) =>
    DROP N<Department>(department_id)
    RETURN "success"

QUERY createCareEvent (event_sequence: I32, timestamp: String, event_type: String, duration_minutes: I32) =>
    care_event <- AddN<CareEvent>({event_sequence: event_sequence, timestamp: timestamp, event_type: event_type, duration_minutes: duration_minutes})
    RETURN care_event

QUERY getCareEvent (care_event_id: ID) =>
    care_event <- N<CareEvent>(care_event_id)
    RETURN care_event

QUERY getAllCareEvents () =>
    care_events <- N<CareEvent>
    RETURN care_events

QUERY updateCareEvent (care_event_id: ID, event_sequence: I32, timestamp: String, event_type: String, duration_minutes: I32) =>
    care_event <- N<CareEvent>(care_event_id)::UPDATE({event_sequence: event_sequence, timestamp: timestamp, event_type: event_type, duration_minutes: duration_minutes})
    RETURN care_event

QUERY deleteCareEvent (care_event_id: ID) =>
    DROP N<CareEvent>(care_event_id)
    RETURN "success"

QUERY addClinicalNote (text: String, timestamp: String, vector: [F64]) =>
    note <- AddV<ClinicalNote>(vector, {text: text, timestamp: timestamp})
    RETURN note

QUERY searchClinicalNotes (vector: [F64], k: I64) =>
    notes <- SearchV<ClinicalNote>(vector, k)
    RETURN notes

QUERY assignDoctorToHospital (doctor_id: ID, hospital_id: ID) =>
    doctor <- N<Doctor>(doctor_id)
    hospital <- N<Hospital>(hospital_id)
    relationship <- AddE<Doctor_WorksAt_Hospital>()::From(doctor)::To(hospital)
    RETURN relationship


QUERY assignDoctorToPatient (doctor_id: ID, patient_id: ID) =>
    doctor <- N<Doctor>(doctor_id)
    patient <- N<Patient>(patient_id)
    relationship <- AddE<Doctor_Treats_Patient>()::From(doctor)::To(patient)
    RETURN relationship


QUERY assignDepartmentToHospital (department_id: ID, hospital_id: ID) =>
    department <- N<Department>(department_id)
    hospital <- N<Hospital>(hospital_id)
    relationship <- AddE<Department_PartOf_Hospital>()::From(department)::To(hospital)
    RETURN relationship

QUERY assignPatientEvent (patient_id: ID, care_event_id: ID) =>
    patient <- N<Patient>(patient_id)
    care_event <- N<CareEvent>(care_event_id)
    relationship <- AddE<Patient_HasEvent_CareEvent>()::From(patient)::To(care_event)
    RETURN relationship

QUERY linkCareEvents (from_event_id: ID, to_event_id: ID, time_gap_minutes: I32) =>
    from_event <- N<CareEvent>(from_event_id)
    to_event <- N<CareEvent>(to_event_id)
    relationship <- AddE<CareEvent_NextEvent_CareEvent>({time_gap_minutes: time_gap_minutes})::From(from_event)::To(to_event)
    RETURN relationship

QUERY assignEventToDepartment (care_event_id: ID, department_id: ID) =>
    care_event <- N<CareEvent>(care_event_id)
    department <- N<Department>(department_id)
    relationship <- AddE<CareEvent_InDepartment_Department>()::From(care_event)::To(department)
    RETURN relationship

QUERY assignDoctorToEvent (doctor_id: ID, care_event_id: ID) =>
    doctor <- N<Doctor>(doctor_id)
    care_event <- N<CareEvent>(care_event_id)
    relationship <- AddE<Doctor_PerformsEvent_CareEvent>()::From(doctor)::To(care_event)
    RETURN relationship

QUERY createDoctorReferral (from_doctor_id: ID, to_doctor_id: ID, referral_reason: String, referral_time: String, urgency: String) =>
    from_doctor <- N<Doctor>(from_doctor_id)
    to_doctor <- N<Doctor>(to_doctor_id)
    referral <- AddE<Doctor_Refers_Doctor>({referral_reason: referral_reason, referral_time: referral_time, urgency: urgency})::From(from_doctor)::To(to_doctor)
    RETURN referral

QUERY addPatientNote (patient_id: ID, text: String, timestamp: String, vector: [F64]) =>
    patient <- N<Patient>(patient_id)
    note <- AddV<ClinicalNote>(vector, {text: text, timestamp: timestamp})
    relationship <- AddE<Patient_HasNote_ClinicalNote>()::From(patient)::To(note)
    RETURN note

QUERY addDoctorNote (doctor_id: ID, text: String, timestamp: String, vector: [F64]) =>
    doctor <- N<Doctor>(doctor_id)
    note <- AddV<ClinicalNote>(vector, {text: text, timestamp: timestamp})
    relationship <- AddE<Doctor_WritesNote_ClinicalNote>()::From(doctor)::To(note)
    RETURN note

QUERY getPatientByName (patient_name: String) =>
    patient <- N<Patient>::WHERE(_::{name}::EQ(patient_name))
    RETURN patient

QUERY getDoctorByName (doctor_name: String) =>
    doctor <- N<Doctor>::WHERE(_::{name}::EQ(doctor_name))
    RETURN doctor

QUERY getHospitalByName (hospital_name: String) =>
    hospital <- N<Hospital>::WHERE(_::{name}::EQ(hospital_name))
    RETURN hospital

QUERY getDepartmentByName (department_name: String) =>
    department <- N<Department>::WHERE(_::{name}::EQ(department_name))
    RETURN department

QUERY getPatientsByAge (min_age: I32, max_age: I32) =>
    patients <- N<Patient>::WHERE(
        AND(
            _::{age}::GTE(min_age),
            _::{age}::LTE(max_age)
        )
    )
    RETURN patients

QUERY getPatientsByBloodType (blood_type: String) =>
    patients <- N<Patient>::WHERE(_::{blood_type}::EQ(blood_type))
    RETURN patients

QUERY getPatientsByMedicalCondition (medical_condition: String) =>
    patients <- N<Patient>::WHERE(_::{medical_condition}::EQ(medical_condition))
    RETURN patients

QUERY getPatientsByInsurance (insurance_provider: String) =>
    patients <- N<Patient>::WHERE(_::{insurance_provider}::EQ(insurance_provider))
    RETURN patients

QUERY getPatientsByBillingRange (min_amount: F64, max_amount: F64) =>
    patients <- N<Patient>::WHERE(
        AND(
            _::{billing_amount}::GTE(min_amount),
            _::{billing_amount}::LTE(max_amount)
        )
    )
    RETURN patients

QUERY getCareEventsByType (event_type: String) =>
    events <- N<CareEvent>::WHERE(_::{event_type}::EQ(event_type))
    RETURN events

QUERY getCareEventsByDuration (min_duration: I32) =>
    events <- N<CareEvent>::WHERE(_::{duration_minutes}::GTE(min_duration))
    RETURN events

QUERY getDoctorsByHospital (hospital_id: ID) =>
    hospital <- N<Hospital>(hospital_id)
    doctors <- hospital::In<Doctor_WorksAt_Hospital>
    RETURN doctors

QUERY getDepartmentsByHospital (hospital_id: ID) =>
    hospital <- N<Hospital>(hospital_id)
    departments <- hospital::In<Department_PartOf_Hospital>
    RETURN departments

QUERY getPatientEvents (patient_id: ID) =>
    patient <- N<Patient>(patient_id)
    events <- patient::Out<Patient_HasEvent_CareEvent>
    RETURN events

QUERY getEventsByDepartment (department_id: ID) =>
    department <- N<Department>(department_id)
    events <- department::In<CareEvent_InDepartment_Department>
    RETURN events

QUERY getDoctorEvents (doctor_id: ID) =>
    doctor <- N<Doctor>(doctor_id)
    events <- doctor::Out<Doctor_PerformsEvent_CareEvent>
    RETURN events

QUERY getDoctorReferrals (doctor_id: ID) =>
    doctor <- N<Doctor>(doctor_id)
    referrals <- doctor::Out<Doctor_Refers_Doctor>
    RETURN referrals

QUERY getPatientNotes (patient_id: ID) =>
    patient <- N<Patient>(patient_id)
    notes <- patient::Out<Patient_HasNote_ClinicalNote>
    RETURN notes

QUERY getDoctorNotes (doctor_id: ID) =>
    doctor <- N<Doctor>(doctor_id)
    notes <- doctor::Out<Doctor_WritesNote_ClinicalNote>
    RETURN notes

QUERY getNextEvents (care_event_id: ID) =>
    event <- N<CareEvent>(care_event_id)
    next_events <- event::Out<CareEvent_NextEvent_CareEvent>
    RETURN next_events

QUERY getPreviousEvents (care_event_id: ID) =>
    event <- N<CareEvent>(care_event_id)
    previous_events <- event::In<CareEvent_NextEvent_CareEvent>
    RETURN previous_events

QUERY getPatientCareSequence (patient_id: ID) =>
    patient <- N<Patient>(patient_id)
    events <- patient::Out<Patient_HasEvent_CareEvent>
    RETURN events

QUERY getHospitalPatients (hospital_id: ID) =>
    hospital <- N<Hospital>(hospital_id)
    departments <- hospital::In<Department_PartOf_Hospital>
    events <- departments::In<CareEvent_InDepartment_Department>
    patients <- events::In<Patient_HasEvent_CareEvent>
    RETURN patients

QUERY getDoctorPatients (doctor_id: ID) =>
    doctor <- N<Doctor>(doctor_id)
    events <- doctor::Out<Doctor_PerformsEvent_CareEvent>
    patients <- events::In<Patient_HasEvent_CareEvent>
    RETURN patients

QUERY getPatientsByDoctor (doctor_name: String) =>
    doctor <- N<Doctor>::WHERE(_::{name}::EQ(doctor_name))
    events <- doctor::Out<Doctor_PerformsEvent_CareEvent>
    patients <- events::In<Patient_HasEvent_CareEvent>
    RETURN patients

QUERY countPatientsByHospital (hospital_id: ID) =>
    hospital <- N<Hospital>(hospital_id)
    departments <- hospital::In<Department_PartOf_Hospital>
    events <- departments::In<CareEvent_InDepartment_Department>
    patient_count <- events::In<Patient_HasEvent_CareEvent>::COUNT
    RETURN patient_count

QUERY getHighUrgencyReferrals () =>
    doctors <- N<Doctor>
    referrals <- doctors::OutE<Doctor_Refers_Doctor>::WHERE(_::{urgency}::EQ("high"))
    RETURN referrals

QUERY getRecentCareEvents (days_back: I32) =>
    events <- N<CareEvent>
    RETURN events

QUERY getPatientsByAdmissionType (admission_type: String) =>
    patients <- N<Patient>::WHERE(_::{admission_type}::EQ(admission_type))
    RETURN patients

QUERY updateDoctorReferral (from_doctor_id: ID, to_doctor_id: ID, referral_reason: String, referral_time: String, urgency: String) =>
    DROP N<Doctor>(from_doctor_id)::OutE<Doctor_Refers_Doctor>
    from_doctor <- N<Doctor>(from_doctor_id)
    to_doctor <- N<Doctor>(to_doctor_id)
    referral <- AddE<Doctor_Refers_Doctor>({referral_reason: referral_reason, referral_time: referral_time, urgency: urgency})::From(from_doctor)::To(to_doctor)
    RETURN referral

QUERY removeDoctorFromHospital (doctor_id: ID, hospital_id: ID) =>
    DROP N<Doctor>(doctor_id)::OutE<Doctor_WorksAt_Hospital>
    RETURN "success"

QUERY removePatientEvent (patient_id: ID, care_event_id: ID) =>
    DROP N<Patient>(patient_id)::OutE<Patient_HasEvent_CareEvent>
    RETURN "success"


QUERY getDoctorTreatsPatientEdges() =>
    edges <- E<Doctor_Treats_Patient>
    RETURN edges

QUERY getDoctorTreatsPatientEdgesByDoctor(doctor_id: ID) =>
    doctor_node <- N<Doctor>(doctor_id)
    edges <- doctor_node::OutE<Doctor_Treats_Patient>
    RETURN edges
