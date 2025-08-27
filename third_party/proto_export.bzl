"""Rule to collect transitive proto sources from proto_library targets."""

def _transitive_proto_sources_impl(ctx):
    """Implementation of the transitive_proto_sources rule."""
    all_sources = depset()
    for dep in ctx.attr.deps:
        if ProtoInfo in dep:
            all_sources = depset(transitive = [all_sources, dep[ProtoInfo].transitive_sources])

    if not ctx.attr.exclude:
        return [DefaultInfo(files = all_sources)]

    # Filter out excluded files
    kept_files = []
    all_files = all_sources.to_list()
    for f in all_files:
        is_excluded = False
        for exclusion_pattern in ctx.attr.exclude:
            # f.short_path is the path relative to the (external) repository root.
            # e.g., "google/protobuf/any.proto"
            if f.short_path.startswith(exclusion_pattern):
                is_excluded = True
                break
        if not is_excluded:
            kept_files.append(f)

    final_sources = depset(kept_files)
    return [DefaultInfo(files = final_sources)]

transitive_proto_sources = rule(
    implementation = _transitive_proto_sources_impl,
    doc = "Collects all transitive .proto files from the given proto_library targets.",
    attrs = {
        "deps": attr.label_list(
            providers = [ProtoInfo],
            mandatory = True,
            doc = "A list of proto_library targets.",
        ),
        "exclude": attr.string_list(
            doc = "A list of path prefixes to exclude from the collected sources.",
        ),
    },
)