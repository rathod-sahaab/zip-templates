use criterion::{criterion_group, criterion_main, Criterion};
use serde_json::json;
use std::hint::black_box;
use tera::Context;
use zip_templates::ZipTemplate;

fn prepare_data() -> (String, serde_json::Value, zip_templates::ZipTemplate) {
    let template = String::from(
        "Hi, {{user.name.first}} — balance: {{account.balance}} USD. Messages: {{meta.count}}.",
    );
    let data = json!({
        "user": { "name": { "first": "Sam" } },
        "account": { "balance": 12.34 },
        "meta": { "count": 5 }
    });

    let parsed = ZipTemplate::parse(&template);
    (template, data, parsed)
}

fn bench_zip_templates_flat(c: &mut Criterion) {
    let (_template, data, parsed) = prepare_data();
    let flat = zip_templates::flatten_json(&data);
    // Simulate already flattened data (clone for realism)
    let already_flat = flat.clone();
    c.bench_function("zip_templates::render_flat", |b| {
        b.iter(|| {
            let out = parsed.render(&already_flat);
            black_box(out);
        })
    });
}

fn bench_zip_templates(c: &mut Criterion) {
    let (_template, data, parsed) = prepare_data();
    c.bench_function("zip_templates::render", |b| {
        b.iter(|| {
            let flat = zip_templates::flatten_json(&data);
            let out = parsed.render(&flat);
            black_box(out);
        })
    });
}

fn bench_tera(c: &mut Criterion) {
    let (template, data, _parsed) = prepare_data();

    // compile template once
    let mut tera = tera::Tera::default();
    tera.add_raw_template("tpl", &template).unwrap();

    c.bench_function("tera::render", |b| {
        b.iter(|| {
            let ctx = Context::from_serialize(&data).unwrap();
            let out = tera.render("tpl", &ctx).unwrap();
            black_box(out);
        })
    });
}

fn bench_simple_replace(c: &mut Criterion) {
    let (template, data, parsed) = prepare_data();

    // prepare map of keys to lookup strings
    let keys: Vec<String> = parsed.placeholders.clone();

    c.bench_function("simple_replace", |b| {
        b.iter(|| {
            let mut out = template.clone();
            for key in &keys {
                // build placeholder token like {{key}}
                let token = format!("{{{{{}}}}}", key);
                // Instead of reusing render we can resolve directly; for simplicity, use a small lookup
                let replacement = {
                    let parts: Vec<&str> = key.split('.').collect();
                    let mut cur = &data;
                    let mut found = None;
                    for p in parts {
                        if cur.is_object() {
                            if let Some(v) = cur.get(p) {
                                cur = v;
                                found = Some(v);
                            } else {
                                found = None;
                                break;
                            }
                        } else {
                            found = None;
                            break;
                        }
                    }
                    if let Some(v) = found {
                        if v.is_string() {
                            v.as_str().unwrap().to_string()
                        } else {
                            v.to_string()
                        }
                    } else {
                        String::new()
                    }
                };
                out = out.replace(&token, &replacement);
            }
            black_box(out);
        })
    });
}

criterion_group!(
    benches,
    bench_zip_templates,
    bench_zip_templates_flat,
    bench_tera,
    bench_simple_replace
);
criterion_main!(benches);
