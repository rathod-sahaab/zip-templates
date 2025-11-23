use criterion::{criterion_group, criterion_main, Criterion};
use mystical_runic::{TemplateContext, TemplateEngine, TemplateValue};
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

fn generate_large_template(
    size_kb: usize,
    placeholders: usize,
) -> (String, Vec<String>, Vec<String>) {
    let mut predefined_map = rustc_hash::FxHashMap::default();
    predefined_map.insert("name", "John Doe");
    predefined_map.insert("city", "New York");
    predefined_map.insert("product", "SuperWidget");
    predefined_map.insert("price", "99.99");
    predefined_map.insert("date", "2025-11-23");
    predefined_map.insert("userId", "a-b-c-d-e-f");
    predefined_map.insert("email", "john.doe@example.com");
    predefined_map.insert("status", "active");
    predefined_map.insert("orderId", "1234567890");
    predefined_map.insert("tracking", "Z9Y8X7W6V5");

    let map_keys: Vec<_> = predefined_map.keys().cloned().collect();
    let keys_to_use: Vec<_> = map_keys.iter().take(placeholders).cloned().collect();

    let mut template =
        String::from("<!DOCTYPE html><html><head><title>Large Template</title></head><body>");
    let lorem_ipsum = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. ";

    let target_len = size_kb * 1024;

    if keys_to_use.is_empty() {
        while template.len() < target_len {
            template.push_str(lorem_ipsum);
        }
    } else {
        // Estimate total insertions to keep density somewhat constant
        let num_insertions = (target_len as f32 / 250.0).ceil() as usize;
        let avg_placeholder_len = (keys_to_use.iter().map(|k| k.len() + 4).sum::<usize>() / keys_to_use.len()).max(1);

        let total_text_len = target_len.saturating_sub(num_insertions * avg_placeholder_len);
        let lorem_chunks_total = (total_text_len as f32 / lorem_ipsum.len() as f32).ceil() as usize;
        let lorem_chunks_per_ph = (lorem_chunks_total as f32 / num_insertions as f32).ceil() as usize;

        let mut current_key_index = 0;
        for _ in 0..num_insertions {
            if template.len() >= target_len {
                break;
            }
            for _ in 0..lorem_chunks_per_ph {
                template.push_str(lorem_ipsum);
            }
            let key = keys_to_use[current_key_index % keys_to_use.len()];
            let ph = format!("{{{{{}}}}}", key);
            template.push_str(&ph);
            current_key_index += 1;
        }
    }

    while template.len() < target_len {
        template.push_str(lorem_ipsum);
    }
    template.push_str("</body></html>");
    template.truncate(target_len);

    let final_keys: Vec<String> = keys_to_use.iter().map(|s| s.to_string()).collect();
    let final_values: Vec<String> = final_keys
        .iter()
        .map(|k| predefined_map.get(k.as_str()).unwrap().to_string())
        .collect();

    (template, final_keys, final_values)
}

fn bench_large_templates(c: &mut Criterion) {
    let sizes = [1, 2, 3, 5, 10, 20, 50, 100];
    let ph_counts = [1, 2, 3, 5, 10];
    for &size in &sizes {
        for &ph_count in &ph_counts {
            let (template, keys, values) = generate_large_template(size, ph_count);
            let parsed = ZipTemplate::parse(&template);
            let mut flat = rustc_hash::FxHashMap::default();
            for (i, key) in keys.iter().enumerate() {
                let ph = format!("{{{{{}}}}}", key);
                flat.insert(ph, values[i].clone());
            }

            // Tera setup
            let mut tera = tera::Tera::default();
            tera.add_raw_template("tpl", &template).unwrap();
            let mut tera_ctx = tera::Context::new();
            for (i, key) in keys.iter().enumerate() {
                tera_ctx.insert(key, &values[i]);
            }

            // Mystical Runic setup
            let mut engine = TemplateEngine::new("template");
            let mut context = TemplateContext::new();
            for (i, key) in keys.iter().enumerate() {
                context.set(key, TemplateValue::String(values[i].clone()));
            }

            let dynamics = values.clone();

            let group_name = format!("large_templates_{}KB_{}ph", size, ph_count);
            let mut group = c.benchmark_group(&group_name);

            group.bench_function("zip_templates::render", |b| {
                b.iter(|| {
                    let out = parsed.render(&flat);
                    black_box(out);
                })
            });

            group.bench_function("zip_templates::render_from_vec", |b| {
                b.iter(|| {
                    let out = parsed.render_from_vec(&dynamics);
                    black_box(out);
                })
            });

            group.bench_function("tera::render", |b| {
                b.iter(|| {
                    let out = tera.render("tpl", &tera_ctx).unwrap();
                    black_box(out);
                })
            });

            group.bench_function("mystical_runic::render", |b| {
                b.iter(|| {
                    let out = engine.render_string(&template, &context).unwrap();
                    black_box(out);
                })
            });

            group.bench_function("simple_replace_flat", |b| {
                b.iter(|| {
                    let mut out = template.clone();
                    for key in &keys {
                        let token = format!("{{{{{}}}}}", key);
                        if let Some(replacement) = flat.get(&token) {
                            out = out.replace(&token, replacement);
                        }
                    }
                    black_box(out);
                })
            });

            group.bench_function("simple_replace", |b| {
                b.iter(|| {
                    let mut out = template.clone();
                    for key in &keys {
                        let token = format!("{{{{{}}}}}", key);
                        if let Some(replacement) = flat.get(&token) {
                            out = out.replace(&token, replacement);
                        }
                    }
                    black_box(out);
                })
            });

            group.finish();
        }
    }
}

fn bench_all_engines(c: &mut Criterion) {
    let (template, data) = prepare_data();
    let parsed = ZipTemplate::parse(&template);
    let flat = zip_templates::flatten_json(&data);
    let already_flat = flat.clone();
    let dynamics = vec!["Sam".to_string(), "12.34".to_string(), "5".to_string()];

    // Tera setup
    let mut tera = tera::Tera::default();
    tera.add_raw_template("tpl", &template).unwrap();

    // Mystical Runic setup
    let mut engine = TemplateEngine::new("template");
    let mut context = TemplateContext::new();
    context.set(
        "user.name.first",
        TemplateValue::String(data["user"]["name"]["first"].as_str().unwrap().to_string()),
    );
    context.set(
        "account.balance",
        TemplateValue::String(data["account"]["balance"].as_f64().unwrap().to_string()),
    );
    context.set(
        "meta.count",
        TemplateValue::String(data["meta"]["count"].as_i64().unwrap().to_string()),
    );

    // Simple Replace setup
    let keys: Vec<String> = parsed.placeholders.clone();

    let mut group = c.benchmark_group("template_engines_compare");

    group.bench_function("flatten then zip_templates::render", |b| {
        b.iter(|| {
            let flat = zip_templates::flatten_json(&data);
            let out = parsed.render(&flat);
            black_box(out);
        })
    });

    group.bench_function("zip_templates::render", |b| {
        b.iter(|| {
            let out = parsed.render(&already_flat);
            black_box(out);
        })
    });

    group.bench_function("zip_templates::render_from_vec", |b| {
        b.iter(|| {
            let out = parsed.render_from_vec(&dynamics);
            black_box(out);
        })
    });

    group.bench_function("tera::render", |b| {
        b.iter(|| {
            let ctx = Context::from_serialize(&data).unwrap();
            let out = tera.render("tpl", &ctx).unwrap();
            black_box(out);
        })
    });

    group.bench_function("mystical_runic::render", |b| {
        b.iter(|| {
            let out = engine.render_string(&template, &context).unwrap();
            black_box(out);
        })
    });

    group.bench_function("simple_replace_flat", |b| {
        b.iter(|| {
            let mut out = template.clone();
            for key in &keys {
                let token = format!("{{{{{}}}}}", key);
                if let Some(replacement) = flat.get(&token) {
                    out = out.replace(&token, replacement);
                }
            }
            black_box(out);
        })
    });

    group.bench_function("simple_replace", |b| {
        b.iter(|| {
            let flat = zip_templates::flatten_json(&data);
            let mut out = template.clone();
            for key in &keys {
                let token = format!("{{{{{}}}}}", key);
                if let Some(replacement) = flat.get(&token) {
                    out = out.replace(&token, replacement);
                }
            }
            black_box(out);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_all_engines, bench_large_templates);
criterion_main!(benches);
