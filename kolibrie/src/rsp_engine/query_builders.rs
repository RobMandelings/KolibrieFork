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
    FROM <http://127.0.0.1:9000/WebGlCity/RDF/SensorRepository.rdf>
    FROM NAMED WINDOW :w1 ON :AarhusWeatherData0 [RANGE {size} STEP {slide}]
    FROM NAMED WINDOW :w2 ON :AarhusTrafficData158505 [RANGE {size} STEP {slide}]
    WHERE {{
      # Weather observations (temp, humidity, wind)
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

      # Traffic congestion observation
      WINDOW :w2 {{
        ?obId4 ssn:observedProperty ?p4 ;
               sao:hasValue ?v4 ;
               ssn:observedBy <AarhusTrafficData158505> .
      }}

      # Optional: type constraints (uncomment if you want them enforced)
      # ?p1 a ct:Temperature .
      # ?p2 a ct:Humidity .
      # ?p3 a ct:WindSpeed .
      # ?p4 a ct:CongestionLevel .
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

      # Second traffic stream (158505)
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