//! One day's osmChange for the merge test: a node moved, a road deleted, a node, road and restriction added, and a
//! name with every kind of XML escape.

pub const CHANGE: &str = r#"<?xml version='1.0' encoding='UTF-8'?>
<osmChange version="0.6" generator="osmium/1.16.0">
  <modify>
    <node id="1000" version="2" timestamp="2026-11-11T10:00:00Z" lat="36.0995" lon="-115.2"/>
  </modify>
  <delete>
    <way id="11" version="3"/>
  </delete>
  <create>
    <node id="5000" version="1" lat="36.1015" lon="-115.2005">
      <tag k="name" v="Jo&amp;Al&apos;s &#x2192; &#169;"/>
    </node>
    <way id="600" version="1">
      <nd ref="5000"/>
      <nd ref="1000"/>
      <tag k="highway" v="residential"/>
    </way>
    <relation id="77" version="1">
      <member type="way" ref="600" role="from"/>
      <member type="node" ref="1000" role="via"/>
      <member type="way" ref="50" role="to"/>
      <tag k="type" v="restriction"/>
    </relation>
  </create>
</osmChange>
"#;
