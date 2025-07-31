// patient node
N::Patient {
    name: String,
    age: I32,
    gender: String,
    blood_type: String,
    medical_condition: String,
    date_of_admission: String,
    discharge_date: String,
    insurance_provider: String,
    billing_amount: F64,
    room_number: String,
    admission_type: String,
    medication: String,
    test_results: String
}

// doctor node
N::Doctor {
    name: String
}

// hospital node
N::Hospital {
    name: String
}

// department node
N::Department {
    name: String
}

// care event node
N::CareEvent {
    event_sequence: I32,
    timestamp: String,
    event_type: String,
    duration_minutes: I32
}

// vector node for clinical note
V::ClinicalNote {
    vector: [F64],
    text: String,
    timestamp: String
}

// doctor works at hospital relationship
E::Doctor_WorksAt_Hospital {
    From: Doctor,
    To: Hospital,
    Properties: {
    }
}

E::Doctor_Treats_Patient {
    From: Doctor,
    To: Patient,
    Properties: {
    }
}

// department part of hospital relationship
E::Department_PartOf_Hospital {
    From: Department,
    To: Hospital,
    Properties: {
    }
}

// patient has event relationship
E::Patient_HasEvent_CareEvent {
    From: Patient,
    To: CareEvent,
    Properties: {
    }
}

// care event sequence relationship
E::CareEvent_NextEvent_CareEvent {
    From: CareEvent,
    To: CareEvent,
    Properties: {
        time_gap_minutes: I32
    }
}

// care event in department relationship
E::CareEvent_InDepartment_Department {
    From: CareEvent,
    To: Department,
    Properties: {
    }
}

// doctor performs event relationship
E::Doctor_PerformsEvent_CareEvent {
    From: Doctor,
    To: CareEvent,
    Properties: {
    }
}

// doctor refers to doctor relationship
E::Doctor_Refers_Doctor {
    From: Doctor,
    To: Doctor,
    Properties: {
        referral_reason: String,
        referral_time: String,
        urgency: String
    }
}

// patient has note relationship
E::Patient_HasNote_ClinicalNote {
    From: Patient,
    To: ClinicalNote,
    Properties: {
    }
}

// doctor writes note relationship
E::Doctor_WritesNote_ClinicalNote {
    From: Doctor,
    To: ClinicalNote,
    Properties: {
    }
}
