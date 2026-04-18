#!/bin/bash
# Merge the build-generated manifest.ttl (plugin definition) with the
# static-lv2-ttl preset manifest that was copied on top of it.
# The preset copy overwrites the original, losing the lv2:Plugin entry.
# This script prepends the plugin definition back into the manifest.

BUNDLE_DIR="$1"
MANIFEST="$BUNDLE_DIR/manifest.ttl"

[ -f "$MANIFEST" ] || { echo "merge-manifest: $MANIFEST not found"; exit 1; }

# If the plugin definition is already present, nothing to do
grep -q 'a lv2:Plugin' "$MANIFEST" && exit 0

# Prepend the plugin definition after the prefix block
sed -i '/^@prefix.*\.$/a\
\
<urn:distrho:vitalium>\
    a lv2:Plugin ;\
    lv2:binary <vitalium-lv2.so> ;\
    rdfs:seeAlso <vitalium-lv2.ttl> .' "$MANIFEST"

# Ensure required prefixes exist
grep -q '@prefix lv2:' "$MANIFEST" || sed -i '1i @prefix lv2:  <http://lv2plug.in/ns/lv2core#> .' "$MANIFEST"
grep -q '@prefix rdfs:' "$MANIFEST" || sed -i '1i @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .' "$MANIFEST"
grep -q '@prefix ui:' "$MANIFEST" || sed -i '1i @prefix ui:   <http://lv2plug.in/ns/extensions/ui#> .' "$MANIFEST"
