use criterion::{criterion_group, criterion_main, Criterion};
use serde_json::json;
use std::hint::black_box;
use tera::Context;
use zip_templates::ZipTemplate;

fn prepare_data() -> (String, serde_json::Value) {
    let template = String::from(
        "Hi, {{user.name.first}} — balance: {{account.balance}} USD. Messages: {{meta.count}}.",
    );
    let data = json!({
        "user": { "name": { "first": "Sam" } },
        "account": { "balance": 12.34 },
        "meta": { "count": 5 }
    });

    (template, data)
}

fn bench_zip_templates_flat(c: &mut Criterion) {
    let (template, data) = prepare_data();
    let parsed = ZipTemplate::from(&template);

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
    let (template, data) = prepare_data();
    let parsed = ZipTemplate::from(&template);

    c.bench_function("zip_templates::render", |b| {
        b.iter(|| {
            let flat = zip_templates::flatten_json(&data);
            let out = parsed.render(&flat);
            black_box(out);
        })
    });
}

fn bench_zip_templates_from_vec(c: &mut Criterion) {
    let (template, _) = prepare_data();
    let parsed = ZipTemplate::from(&template);

    let dynamics = vec!["Sam".to_string(), "12.34".to_string(), "5".to_string()];
    c.bench_function("zip_templates::render_from_vec_smart", |b| {
        b.iter(|| {
            let out = parsed.render_from_vec(&dynamics);
            black_box(out);
        })
    });
}

fn bench_tera(c: &mut Criterion) {
    let (template, data) = prepare_data();

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

fn bench_simple_replace_flat(c: &mut Criterion) {
    let (template, data) = prepare_data();

    /* using zip_templates parse to get list of tokens, not counted in bench */
    let parsed = ZipTemplate::from(&template);

    let flat = zip_templates::flatten_json(&data);

    // prepare map of keys to lookup strings
    let keys: Vec<String> = parsed.placeholders.clone();

    c.bench_function("simple_replace_flat", |b| {
        b.iter(|| {
            let mut out = template.clone();
            for key in &keys {
                // build placeholder token like {{key}}
                let token = format!("{{{{{}}}}}", key);
                // Instead of reusing render we can resolve directly; for simplicity, use a small lookup
                if let Some(replacement) = flat.get(&token) {
                    out = out.replace(&token, replacement);
                }
            }
            black_box(out);
        })
    });
}

fn bench_simple_replace(c: &mut Criterion) {
    let (template, data) = prepare_data();
    let parsed = ZipTemplate::from(&template);

    // prepare map of keys to lookup strings
    let keys: Vec<String> = parsed.placeholders.clone();

    c.bench_function("simple_replace", |b| {
        b.iter(|| {
            let flat = zip_templates::flatten_json(&data);
            let mut out = template.clone();
            for key in &keys {
                // build placeholder token like {{key}}
                let token = format!("{{{{{}}}}}", key);
                // Instead of reusing render we can resolve directly; for simplicity, use a small lookup
                if let Some(replacement) = flat.get(&token) {
                    out = out.replace(&token, replacement);
                }
            }
            black_box(out);
        })
    });
}

criterion_group!(
    benches,
    bench_zip_templates,
    bench_zip_templates_flat,
    bench_zip_templates_from_vec,
    bench_tera,
    bench_simple_replace,
    bench_simple_replace_flat,
);
criterion_main!(benches);
