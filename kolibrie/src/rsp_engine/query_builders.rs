use prototypes::WindowParams;

pub fn build_q1_query(params: &WindowParams) -> String {
    format!(
        r#"
    PREFIX ses: <http://www.insight-centre.org/dataset/SampleEventService#>
    PREFIX ssn: <http://purl.oclc.org/NET/ssnx/ssn#>
    PREFIX sao: <http://purl.oclc.org/NET/sao/>
    PREFIX ct:  <http://www.insight-centre.org/citytraffic#>
    REGISTER RSTREAM <http://out/stream> AS
    SELECT ?obId1 ?v1
    FROM NAMED WINDOW :w1 ON :AarhusTrafficData158505 [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w2 ON :AarhusTrafficData182955 [RANGE {size} STEP {slide}]
    WHERE {{
      WINDOW :w1 {{
        ?obId1 ssn:observedProperty ?p1 ;
               sao:hasValue ?v1 ;
               ssn:observedBy <AarhusTrafficData158505> .
      }}
      WINDOW :w2 {{
        ?obId2 ssn:observedProperty ?p2 ;
               sao:hasValue ?v2 ;
               ssn:observedBy <AarhusTrafficData182955> .
      }}
    }}"#,
        size = params.size,
        slide = params.slide,
    )
}

pub fn build_q2_query(params: &WindowParams) -> String {
    format!(
        r#"
    PREFIX ses: <http://www.insight-centre.org/dataset/SampleEventService#>
    PREFIX ssn: <http://purl.oclc.org/NET/ssnx/ssn#>
    PREFIX sao: <http://purl.oclc.org/NET/sao/>
    PREFIX ct:  <http://www.insight-centre.org/citytraffic#>
    REGISTER RSTREAM <http://out/stream> AS
    SELECT ?obId1 ?obId2 ?obId3 ?obId4 ?v1 ?v2 ?v3 ?v4
    FROM NAMED WINDOW :w1 ON :AarhusWeatherData0 [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w2 ON :AarhusTrafficData182955 [RANGE {size} STEP {slide}]
    WHERE {{
      WINDOW :w1 {{
        ?obId1 ssn:observedProperty ?p1 ;
               sao:hasValue ?v1 ;
               ssn:observedBy <AarhusWeatherData0> .
        ?obId2 ssn:observedProperty ?p2 ;
               sao:hasValue ?v2 ;
               ssn:observedBy <AarhusWeatherData0> .
        ?obId3 ssn:observedProperty ?p3 ;
               sao:hasValue ?v3 ;
               ssn:observedBy <AarhusWeatherData0> .
      }}
      WINDOW :w2 {{
        ?obId4 ssn:observedProperty ?p4 ;
               sao:hasValue ?v4 ;
               ssn:observedBy <AarhusTrafficData182955> .
      }}
    }}"#,
        size = params.size,
        slide = params.slide,
    )
}

pub fn build_q3_query(params: &WindowParams) -> String {
    format!(
        r#"
    PREFIX ses: <http://www.insight-centre.org/dataset/SampleEventService#>
    PREFIX ssn: <http://purl.oclc.org/NET/ssnx/ssn#>
    PREFIX sao: <http://purl.oclc.org/NET/sao/>
    PREFIX ct:  <http://www.insight-centre.org/citytraffic#>
    REGISTER RSTREAM <http://out/stream> AS
    SELECT ?obId1 ?obId3 ?v1 ?v3 (((?v1 + ?v3) / 2) AS ?avgCongest)
    FROM <http://127.0.0.1:9000/WebGlCity/RDF/SensorRepository.rdf>
    FROM NAMED WINDOW :w1 ON :AarhusTrafficData182955 [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w2 ON :AarhusTrafficData158505 [RANGE {size} STEP {slide}]
    WHERE {{
      # Property types
      ?p1 a ct:CongestionLevel .
      ?p3 a ct:CongestionLevel .

      # First traffic stream (182955)
      WINDOW :w1 {{
        ?obId1 a ?ob ;
               ssn:observedProperty ?p1 ;
               sao:hasValue ?v1 ;
               ssn:observedBy <AarhusTrafficData182955> .
      }}
      WINDOW :w2 {{
        ?obId3 a ?ob ;
               ssn:observedProperty ?p3 ;
               sao:hasValue ?v3 ;
               ssn:observedBy <AarhusTrafficData158505> .
      }}
    }}"#,
        size = params.size,
        slide = params.slide,
    )
}

pub fn build_q4_query(params: &WindowParams) -> String {
    format!(
        r#"
    PREFIX ses: <http://www.insight-centre.org/dataset/SampleEventService#>
    PREFIX ssn: <http://purl.oclc.org/NET/ssnx/ssn#>
    PREFIX sao: <http://purl.oclc.org/NET/sao/>
    PREFIX ct:  <http://www.insight-centre.org/citytraffic#>
    REGISTER RSTREAM <http://out/stream> AS
    SELECT ?evtId ?title ?node ?obId2 ?lat2 ?lon2 ?lat1 ?lon1
    FROM <http://127.0.0.1:9000/WebGlCity/RDF/AarhusCulturalEvents.rdf>
    FROM NAMED WINDOW :w1 ON :UserLocationService [RANGE {size} STEP {slide}]
    WHERE {{
      ?evtId a sao:Point ;
             ssn:featureOfInterest ?foi ;
             sao:value ?title .
      ?foi  ct:hasFirstNode ?node .
      ?node ct:hasLatitude ?lat1 ;
            ct:hasLongitude ?lon1 .

      WINDOW :w1 {{
        ?obId2 a ssn:Observation ;
               ssn:observedProperty ?p2 ;
               sao:hasValue ?v2 ;
               ssn:observedBy <UserLocationService> .
        ?v2 ct:hasLatitude ?lat2 ;
            ct:hasLongitude ?lon2 .
      }}

      FILTER (((?lat2 - ?lat1) * (?lat2 - ?lat1)
            +  (?lon2 - ?lon1) * (?lon2 - ?lon1)) < 0.1)
    }}"#,
        size = params.size,
        slide = params.slide,
    )
}

pub fn build_q5_query(params: &WindowParams) -> String {
    format!(
        r#"
    PREFIX ses: <http://www.insight-centre.org/dataset/SampleEventService#>
    PREFIX ssn: <http://purl.oclc.org/NET/ssnx/ssn#>
    PREFIX sao: <http://purl.oclc.org/NET/sao/>
    PREFIX ct:  <http://www.insight-centre.org/citytraffic#>
    REGISTER RSTREAM <http://out/stream> AS
    SELECT ?evtId ?title ?obId2 ?lat2 ?lon2
    FROM <http://127.0.0.1:9000/WebGlCity/RDF/AarhusCulturalEvents.rdf>
    FROM <http://127.0.0.1:9000/WebGlCity/RDF/SensorRepository.rdf>
    FROM NAMED WINDOW :w1 ON :AarhusTrafficData158505 [RANGE {size} STEP {slide}]
    WHERE {{
      ?p2 a ct:CongestionLevel ;
          ssn:isPropertyOf ?foi2 .
      ?foi2 ct:hasStartLatitude ?lat2 ;
            ct:hasStartLongitude ?lon2 .

      {{
        ?evtId a ?ob ;
               ssn:featureOfInterest ?foi ;
               sao:value ?title .
        ?foi ct:hasFirstNode ?node .
        ?node ct:hasLatitude ?lat1 ;
              ct:hasLongitude ?lon1 .
      }}

      WINDOW :w1 {{
        ?obId2 a ?ob ;
               ssn:observedProperty ?p2 ;
               sao:hasValue ?v2 ;
               ssn:observedBy <AarhusTrafficData158505> .
      }}

      FILTER (((?lat2 - ?lat1) * (?lat2 - ?lat1)
            +  (?lon2 - ?lon1) * (?lon2 - ?lon1)) < 0.1)
    }}"#,
        size = params.size,
        slide = params.slide,
    )
}

pub fn build_q6_query(params: &WindowParams) -> String {
    format!(
        r#"
    PREFIX ses: <http://www.insight-centre.org/dataset/SampleEventService#>
    PREFIX ssn: <http://purl.oclc.org/NET/ssnx/ssn#>
    PREFIX sao: <http://purl.oclc.org/NET/sao/>
    PREFIX ct:  <http://www.insight-centre.org/citytraffic#>
    REGISTER RSTREAM <http://out/stream> AS
    SELECT ?obId1 ?obId2 ?lat1 ?lon1 ?lat2 ?lon2
    FROM <http://127.0.0.1:9000/WebGlCity/RDF/SensorRepository.rdf>
    FROM NAMED WINDOW :w1 ON :AarhusParkingDataKALKVAERKSVEJ [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w2 ON :UserLocationService [RANGE {size} STEP {slide}]
    WHERE {{
      ?p1 a ct:ParkingVacancy ;
          ssn:isPropertyOf ?foi1 .
      ?foi1 ct:hasStartLatitude ?lat1 ;
            ct:hasStartLongitude ?lon1 .

      WINDOW :w1 {{
        ?obId1 a ?ob ;
               ssn:observedProperty ?p1 ;
               sao:hasValue ?v1 ;
               ssn:observedBy <AarhusParkingDataKALKVAERKSVEJ> .
      }}

      WINDOW :w2 {{
        ?obId2 a ?ob ;
               sao:hasValue ?v2 ;
               ssn:observedBy <UserLocationService> .
        ?v2 ct:hasLatitude ?lat2 ;
            ct:hasLongitude ?lon2 .
      }}
    }}"#,
        size = params.size,
        slide = params.slide,
    )
}

pub fn build_q7_query(params: &WindowParams) -> String {
    format!(
        r#"
    PREFIX ses: <http://www.insight-centre.org/dataset/SampleEventService#>
    PREFIX ssn: <http://purl.oclc.org/NET/ssnx/ssn#>
    PREFIX sao: <http://purl.oclc.org/NET/sao/>
    PREFIX ct:  <http://www.insight-centre.org/citytraffic#>
    REGISTER RSTREAM <http://out/stream> AS
    SELECT ?obId1 ?obId2 ?v1 ?v2
    FROM <http://127.0.0.1:9000/WebGlCity/RDF/SensorRepository.rdf>
    FROM NAMED WINDOW :w1 ON :AarhusParkingDataKALKVAERKSVEJ [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w2 ON :AarhusParkingDataSKOLEBAKKEN [RANGE {size} STEP {slide}]
    WHERE {{
      ?p1 a ct:ParkingVacancy .
      ?p2 a ct:ParkingVacancy .

      WINDOW :w1 {{
        ?obId1 a ?ob ;
               ssn:observedProperty ?p1 ;
               sao:hasValue ?v1 ;
               ssn:observedBy <AarhusParkingDataKALKVAERKSVEJ> .
      }}

      WINDOW :w2 {{
        ?obId2 a ?ob ;
               ssn:observedProperty ?p2 ;
               sao:hasValue ?v2 ;
               ssn:observedBy <AarhusParkingDataSKOLEBAKKEN> .
      }}

      FILTER (?v1 < 1 || ?v2 < 1)
    }}"#,
        size = params.size,
        slide = params.slide,
    )
}

pub fn build_q8_query(params: &WindowParams) -> String {
    format!(
        r#"
    PREFIX ses: <http://www.insight-centre.org/dataset/SampleEventService#>
    PREFIX ssn: <http://purl.oclc.org/NET/ssnx/ssn#>
    PREFIX sao: <http://purl.oclc.org/NET/sao/>
    PREFIX ct:  <http://www.insight-centre.org/citytraffic#>
    REGISTER RSTREAM <http://out/stream> AS
    SELECT ?obId1 ?obId2 ?v1 ?v2
    FROM <http://127.0.0.1:9000/WebGlCity/RDF/SensorRepository.rdf>
    FROM <http://127.0.0.1:9000/WebGlCity/RDF/AarhusLibraryEvents.rdf>
    FROM NAMED WINDOW :w1 ON :AarhusParkingDataKALKVAERKSVEJ [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w2 ON :AarhusParkingDataSKOLEBAKKEN [RANGE {size} STEP {slide}]
    WHERE {{
      ?p1 a ct:ParkingVacancy .
      ?p2 a ct:ParkingVacancy .

      ?evtId ssn:featureOfInterest ?foi .
      ?foi ct:hasFirstNode ?node .
      ?node ct:hasLatitude ?lat1 ;
            ct:hasLongitude ?lon1 .
      ?evtId sao:value ?title .

      WINDOW :w1 {{
        ?obId1 ssn:observedProperty ?p1 ;
               sao:hasValue ?v1 ;
               ssn:observedBy <AarhusParkingDataKALKVAERKSVEJ> .
      }}

      WINDOW :w2 {{
        ?obId2 ssn:observedProperty ?p2 ;
               sao:hasValue ?v2 ;
               ssn:observedBy <AarhusParkingDataSKOLEBAKKEN> .
      }}

      FILTER (?v1 > 0 || ?v2 > 0)
    }}"#,
        size = params.size,
        slide = params.slide,
    )
}

pub fn build_q9_query(params: &WindowParams) -> String {
    format!(
        r#"
    PREFIX ses: <http://www.insight-centre.org/dataset/SampleEventService#>
    PREFIX ssn: <http://purl.oclc.org/NET/ssnx/ssn#>
    PREFIX sao: <http://purl.oclc.org/NET/sao/>
    PREFIX ct:  <http://www.insight-centre.org/citytraffic#>
    REGISTER RSTREAM <http://out/stream> AS
    SELECT ?obId1 ?obId2 ?v1 ?v2
    FROM <http://127.0.0.1:9000/WebGlCity/RDF/AarhusCulturalEvents.rdf>
    FROM <http://127.0.0.1:9000/WebGlCity/RDF/SensorRepository.rdf>
    FROM NAMED WINDOW :w1 ON :AarhusParkingDataKALKVAERKSVEJ [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w2 ON :AarhusParkingDataSKOLEBAKKEN [RANGE {size} STEP {slide}]
    WHERE {{
      ?p1 a ct:ParkingVacancy .
      ?p2 a ct:ParkingVacancy .

      ?evtId a ?ob ;
             ssn:featureOfInterest ?foi ;
             sao:value ?title .
      ?foi ct:hasFirstNode ?node .
      ?node ct:hasLatitude ?lat1 ;
            ct:hasLongitude ?lon1 .

      WINDOW :w1 {{
        ?obId1 a ?ob ;
               ssn:observedProperty ?p1 ;
               sao:hasValue ?v1 ;
               ssn:observedBy <AarhusParkingDataKALKVAERKSVEJ> .
      }}

      WINDOW :w2 {{
        ?obId2 a ?ob ;
               ssn:observedProperty ?p2 ;
               sao:hasValue ?v2 ;
               ssn:observedBy <AarhusParkingDataSKOLEBAKKEN> .
      }}
    }}"#,
        size = params.size,
        slide = params.slide,
    )
}

pub fn build_q10_query(params: &WindowParams) -> String {
    format!(
        r#"
    PREFIX ses: <http://www.insight-centre.org/dataset/SampleEventService#>
    PREFIX ssn: <http://purl.oclc.org/NET/ssnx/ssn#>
    PREFIX sao: <http://purl.oclc.org/NET/sao/>
    PREFIX ct:  <http://www.insight-centre.org/citytraffic#>
    REGISTER RSTREAM <http://out/stream> AS
    SELECT ?obId1 ?obId2 ((?v1+?v2) AS ?sumOfAPI)
    FROM NAMED WINDOW :w1 ON :AarhusPollutionData201399 [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w2 ON :AarhusPollutionData184892 [RANGE {size} STEP {slide}]
    FROM <http://127.0.0.1:9000/WebGlCity/RDF/SensorRepository.rdf>
    WHERE {{
      WINDOW :w1 {{
        ?obId1 a ?ob ;
               ssn:observedProperty ?p1 ;
               sao:hasValue ?v1 ;
               ssn:observedBy <AarhusPollutionData201399> .
      }}

      WINDOW :w2 {{
        ?obId2 a ?ob ;
               ssn:observedProperty ?p2 ;
               sao:hasValue ?v2 ;
               ssn:observedBy <AarhusPollutionData184892> .
      }}
    }}"#,
        size = params.size,
        slide = params.slide,
    )
}

pub fn build_q10_5_query(params: &WindowParams) -> String {
    format!(
        r#"
    PREFIX ses: <http://www.insight-centre.org/dataset/SampleEventService#>
    PREFIX ssn: <http://purl.oclc.org/NET/ssnx/ssn#>
    PREFIX sao: <http://purl.oclc.org/NET/sao/>
    PREFIX ct:  <http://www.insight-centre.org/citytraffic#>
    REGISTER RSTREAM <http://out/stream> AS
    SELECT ?obId1 ?obId2 ?obId3 ?obId4 ?obId5
    FROM NAMED WINDOW :w1 ON :AarhusPollutionData182955 [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w2 ON :AarhusPollutionData158505 [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w3 ON :AarhusPollutionData206502 [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w4 ON :AarhusPollutionData179093 [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w5 ON :AarhusPollutionData195843 [RANGE {size} STEP {slide}]
    FROM <http://127.0.0.1:9000/WebGlCity/RDF/SensorRepository.rdf>
    WHERE {{
      WINDOW :w1 {{
        ?obId1 ssn:observedProperty ?p1 ;
               sao:hasValue ?v1 ;
               ssn:observedBy <AarhusPollutionData182955> .
      }}

      WINDOW :w2 {{
        ?obId2 ssn:observedProperty ?p2 ;
               sao:hasValue ?v2 ;
               ssn:observedBy <AarhusPollutionData158505> .
      }}

      WINDOW :w3 {{
        ?obId3 ssn:observedProperty ?p3 ;
               sao:hasValue ?v3 ;
               ssn:observedBy <AarhusPollutionData206502> .
      }}

      WINDOW :w4 {{
        ?obId4 ssn:observedProperty ?p4 ;
               sao:hasValue ?v4 ;
               ssn:observedBy <AarhusPollutionData179093> .
      }}

      WINDOW :w5 {{
        ?obId5 ssn:observedProperty ?p5 ;
               sao:hasValue ?v5 ;
               ssn:observedBy <AarhusPollutionData195843> .
      }}
    }}"#,
            size = params.size,
            slide = params.slide,
        )
}

pub fn build_q10_8_query(params: &WindowParams) -> String {
    format!(
        r#"
    PREFIX ses: <http://www.insight-centre.org/dataset/SampleEventService#>
    PREFIX ssn: <http://purl.oclc.org/NET/ssnx/ssn#>
    PREFIX sao: <http://purl.oclc.org/NET/sao/>
    PREFIX ct:  <http://www.insight-centre.org/citytraffic#>
    REGISTER RSTREAM <http://out/stream> AS
    SELECT ?obId1 ?obId2 ?obId3 ?obId4 ?obId5 ?obId6 ?obId7 ?obId8
    FROM NAMED WINDOW :w1 ON :AarhusPollutionData182955 [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w2 ON :AarhusPollutionData158505 [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w3 ON :AarhusPollutionData206502 [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w4 ON :AarhusPollutionData179093 [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w5 ON :AarhusPollutionData195843 [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w6 ON :AarhusPollutionData206237 [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w7 ON :AarhusPollutionData195204 [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w8 ON :AarhusPollutionData204300 [RANGE {size} STEP {slide}]
    FROM <http://127.0.0.1:9000/WebGlCity/RDF/SensorRepository.rdf>
    WHERE {{
      WINDOW :w1 {{
        ?obId1 ssn:observedProperty ?p1 ;
               sao:hasValue ?v1 ;
               ssn:observedBy <AarhusPollutionData182955> .
      }}

      WINDOW :w2 {{
        ?obId2 ssn:observedProperty ?p2 ;
               sao:hasValue ?v2 ;
               ssn:observedBy <AarhusPollutionData158505> .
      }}

      WINDOW :w3 {{
        ?obId3 ssn:observedProperty ?p3 ;
               sao:hasValue ?v3 ;
               ssn:observedBy <AarhusPollutionData206502> .
      }}

      WINDOW :w4 {{
        ?obId4 ssn:observedProperty ?p4 ;
               sao:hasValue ?v4 ;
               ssn:observedBy <AarhusPollutionData179093> .
      }}

      WINDOW :w5 {{
        ?obId5 ssn:observedProperty ?p5 ;
               sao:hasValue ?v5 ;
               ssn:observedBy <AarhusPollutionData195843> .
      }}

      WINDOW :w6 {{
        ?obId6 ssn:observedProperty ?p6 ;
               sao:hasValue ?v6 ;
               ssn:observedBy <AarhusPollutionData206237> .
      }}

      WINDOW :w7 {{
        ?obId7 ssn:observedProperty ?p7 ;
               sao:hasValue ?v7 ;
               ssn:observedBy <AarhusPollutionData195204> .
      }}

      WINDOW :w8 {{
        ?obId8 ssn:observedProperty ?p8 ;
               sao:hasValue ?v8 ;
               ssn:observedBy <AarhusPollutionData204300> .
      }}
    }}"#,
        size = params.size,
        slide = params.slide,
    )
}