use criterion::{Criterion, black_box, criterion_group, criterion_main};
use forge_kit::parser::parse; // Adjust this import based on your actual crate name

fn bench_parser(c: &mut Criterion) {
    // 1. Simple Text (Minimal overhead test)
    c.bench_function("parse_simple_text", |b| {
        b.iter(|| {
            parse(black_box(
                "code: `Hello, world! This is just plain text content.`",
            ))
        })
    });

    // 2. Standard Function Call
    c.bench_function("parse_function_call", |b| {
        b.iter(|| parse(black_box("code: `$get[user_balance;123456789]`")))
    });

    // 3. Deep Nesting (Tests recursion and stack efficiency)
    let nested_input = "code: `$first[$second[$third[$fourth[$fifth[final]]]]]`".repeat(5);
    c.bench_function("parse_deep_nesting", |b| {
        b.iter(|| parse(black_box(&nested_input)))
    });

    // 4. Large Script
    let large_script = r#"
        import { BaseCommand } from "@tryforge/forgescript";
        export default new BaseCommand({
          type: "messageCreate",
          name: "transform",
          aliases: ["cr7"],
          code: `
          $onlyIf[$username==butwhylezi;]
          $let[user;$authorID]
        
          $c[ Constants ]
          $let[size;$default[$message[0];10]]
          $let[frames;$default[$message[1];24]]
          $let[srcImg;$userAvatar[$get[user]]]
          $let[trgImg;https://images2.imgbox.com/9d/32/fyedGEQR_o.jpg]
        
          $c[ Send loading message ]
          $let[loadingMsg;$sendMessage[$channelID;Transforming into CR7 ($get[size]x$get[size]@$get[frames]);true]]
        
          $c[ util ]
          $fn[update;$return[$!editMessage[$channelID;$get[loadingMsg];$getMessage[$channelID;$get[loadingMsg];content]\n- $env[msg] ($round[$divide[$executionTime;1000];2]s)]];msg]
          $fn[lerp;$return[$math[$env[a]+($env[b]-$env[a])*$env[t]]];a;b;t]
          $fn[key;$return[$math[$env[r]*0.3+$env[g]*0.59+$env[b]*0.11]];r;g;b]
        
          $c[ Extract Source Image Pixels ]
          $arrayCreate[srcPixels]
          $createCanvas[srcCanvas;$get[size];$get[size];
            $drawImage[;$get[srcImg];0;0;$get[size];$get[size]]
            $loop[$get[size];
              $let[y;$sub[$env[y];1]]
              $loop[$get[size];
                $let[x;$sub[$env[x];1]]
                $let[i;$math[($get[y] * $get[size] + $get[x]) * 4]]
                $jsonLoad[rgba;$getPixels[;$get[x];$get[y];1;1;Rgba]]
                $jsonLoad[pixel;{}]
                $jsonSet[pixel;x;$get[x]] $jsonSet[pixel;y;$get[y]]
                $jsonSet[pixel;r;$env[rgba;0]] $jsonSet[pixel;g;$env[rgba;1]] $jsonSet[pixel;b;$env[rgba;2]] $jsonSet[pixel;a;$env[rgba;3]]
                $jsonSet[pixel;key;$callFn[key;$env[rgba;0];$env[rgba;1];$env[rgba;2]]]
                $arrayPushJSON[srcPixels;$jsonStringify[pixel]]
              ;x;true]
            ;y;true]
          ]
          $callfn[update;First Goal at $trunc[$divide[$executionTime;780.5]]']
        
          $c[ Extract Target Positions ]
          $arrayCreate[trgPositions]
          $createCanvas[trgCanvas;$get[size];$get[size];
            $drawImage[;$get[trgImg];0;0;$get[size];$get[size]]
            $loop[$get[size];
              $let[y;$sub[$env[y];1]]
              $loop[$get[size];
                $let[x;$sub[$env[x];1]]
                $let[i;$math[($get[y] * $get[size] + $get[x]) * 4]]
                $jsonLoad[rgba;$getPixels[;$get[x];$get[y];1;1;Rgba]]
                $jsonLoad[pixel;{}]
                $jsonSet[pixel;x;$get[x]] $jsonSet[pixel;y;$get[y]]
                $jsonSet[pixel;key;$callFn[key;$env[rgba;0];$env[rgba;1];$env[rgba;2]]]
                $arrayPushJSON[trgPositions;$jsonStringify[pixel]]
              ;x;true]
            ;y;true]
          ]
          $callfn[update;Second Goal at $trunc[$divide[$executionTime;880.5]]']
        
          $c[ Sort both by color similarity ]
          $arrayAdvancedSort[srcPixels;a;b;$return[$sub[$env[a;key];$env[b;key]]];srcPixels]
          $arrayAdvancedSort[trgPositions;a;b;$return[$sub[$env[a;key];$env[b;key]]];trgPositions]
          $callFn[update;bro is the goat]
        
          $c[ Moing pixels ]
          $let[i;0]
          $arrayMap[srcPixels;p;
            $jsonLoad[m;{}]
            $jsonSet[m;startX;$env[p;x]] $jsonSet[m;startY;$env[p;y]]
            $jsonSet[m;endX;$env[trgPositions;$get[i];x]] $jsonSet[m;endY;$env[trgPositions;$get[i];y]]
            $jsonSet[m;r;$env[p;r]] $jsonSet[m;g;$env[p;g]] $jsonSet[m;b;$env[p;b]] $jsonSet[m;a;$env[p;a]]
            $letSum[i;1]
            $return[$env[m]]
          ;moving]
          $callfn[update;*does suii*]
        
          $c[ GIF ]
          $newGIFEncoder[gif;$get[size];$get[size];;
            $setGIFEncoderLoops[;-1]
          ]
          $createCanvas[frame;$get[size];$get[size]]
          $loop[$get[frames];
            $let[frame;$sub[$env[frame];1]]
            $let[t;$divide[$get[frame];$get[frames]]]
            $let[t;$math[$get[t]*$get[t]*(3-2*$get[t])]]
            $drawRect[frame;clear;black;0;0;$get[size];$get[size]]
            $arrayForEach[moving;p;
              $let[x;$round[$callFn[lerp;$env[p;startX];$env[p;endX];$get[t]]]]
              $let[y;$round[$callFn[lerp;$env[p;startY];$env[p;endY];$get[t]]]]
              $drawRect[frame;fill;rgba($env[p;r], $env[p;g], $env[p;b], $divide[$env[p;a];255]);$get[x];$get[y];1;1]
            ]
            $addFrame[gif;canvas://frame]
          ;frame;true]
        
        
          $sendMessage[$channelID;
            # SUII
            $attachCanvas[trgCanvas]
            $attachGIF[gif]
          ]
          `,
        });
    "#;

    c.bench_function("parse_large_script", |b| {
        b.iter(|| parse(black_box(&large_script)))
    });

    c.bench_function("parse_many_arguments", |b| {
        let input = format!("code: `$func[{} ]`", "arg;".repeat(100));
        b.iter(|| parse(black_box(&input)))
    });
    c.bench_function("parse_escape_heavy", |b| {
        let input = "code: `".to_string() + &"\\$text\\[\\;\\`".repeat(50) + "`";
        b.iter(|| parse(black_box(&input)))
    });
    c.bench_function("parse_highly_fragmented", |b| {
        let input = "text code: `$func[]` text ${ js } ".repeat(20);
        b.iter(|| parse(black_box(&input)))
    });
}

criterion_group!(benches, bench_parser);
criterion_main!(benches);
