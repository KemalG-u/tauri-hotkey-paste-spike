> 🧪 **Durum: deney, arşivlendi.** Referans olarak duruyor; bakımı yapılmıyor.

# tauri-hotkey-paste-spike

Windows için iki küçük Tauri v2 denemesi, tek soruya cevap arar: **bir genel kısayol (bas-konuş-bırak tarzı) odaktaki herhangi bir pencereye metni güvenilir biçimde yerleştirebilir mi?** Metin, sabit bir Türkçe test cümlesidir (aksanlı harfler, sayı, para, yüzde); yani deneme, ASCII dışı karakterlerin yolda bozulmadığını da sınar. Konuşma tanıma yoktur.

Neden var: yazar, dikte tarzı bir aracın son halkasını (kısayol basılı = kayıt, bırakılınca = metni yerleştir) kabuk seçmeden önce kanıtlamak istedi. Kod bu kanıttır; aynı yaklaşımı tartan biri için referans olarak duruyor.

## İçerik

| Yol | Ne yapar |
|---|---|
| `src/main.rs`, `Cargo.toml`, `tauri.conf.json`, `frontend/` | En küçük deneme (yaklaşık 150 satır Rust). `Ctrl+Alt+D`: basış kaydedilir, bırakınca pano zinciri çalışır: panoyu sakla, test metnini yaz, sıkışmış değiştirici tuşları bırakıp `SendInput` ile `Ctrl+V` gönder, bekle, panoyu geri yükle. Gizli pencere, yalnız günlük dosyası. |
| `placement-variants/` | Küçük pencereli ikinci deneme, karşılaştırılacak iki yerleştirme yöntemiyle: `Ctrl+Alt+D` = pano + `Ctrl+V` (pano sahiplik ve geri yükleme kontrolüyle), `Ctrl+Alt+U` = `enigo` ile `KEYEVENTF_UNICODE` yazımı (panoya dokunmaz). Düğmeler aynı yöntemleri tetikler ve günlüğü gösterir. Crate ve kütüphane adı eski adıyla `cp3-app` / `cp3_app_lib` kalmıştır. |

## Hızlı başlangıç

Gereksinim: WebView2 olan Windows 10/11 ve Rust. Varyant, GNU araç zincirini `placement-variants/src-tauri/rust-toolchain.toml` ile sabitler (nedeni `Cargo.toml` yorumunda); MSVC için o dosyayı silin.

```
# en küçük deneme (depo kökü)
cargo run
# günlük: çalışma klasöründe events.log ya da SPIKE_TAURI_LOG yolu

# iki yöntemli varyant
cd placement-variants
npm install
npx tauri dev
# günlük: çalışma klasöründe placement-log.jsonl ya da PLACEMENT_SPIKE_LOG yolu
```

Herhangi bir metin alanına tıklayın, sonra kısayola basıp bırakın. İkisini aynı anda çalıştırmayın: ikisi de `Ctrl+Alt+D` kaydeder.

Günlük, önceki panonun içeriğini değil yalnız uzunluğunu yazar.

## Sınırlar ve doğrulanmayanlar

- **Bu sürüm hazırlık sırasında derlenmedi ve çalıştırılmadı.** Hazırlık makinesinde Rust yoktu; bu yüzden test ve CI yoktur. Yazarın kendi çalıştırma günlükleri (pano içeriği taşıdığı için yayımlanmadı) tek bir Windows makinesinde kısayol kaydı, basış/bırakış olayları ve tamamlanan yapıştır-geri yükle turlarını gösteriyordu. Bunu tek makine gözlemi sayın, garanti değil.
- Yalnız Windows (`SendInput`, `windows-sys`). Tek klavye düzeni, tek makine; yönetici yetkili pencere ve uzak masaüstü denenmedi.
- Yapıştırma körlemedir: hedef uygulamanın metni gerçekten alıp almadığı anlaşılamaz (bunun için UI Automation gerekir). Pano geri yükleme sabit gecikme kullanır, yavaş uygulamalarla yarışabilir.
- Bazı uygulamalar sentetik `Ctrl+V`'yi yok sayar; yetkisiz süreçten yetkili pencerelere gönderim engellenir.
- Kod bir denemedir: hatalar günlüğe yazılır, ele alınmaz; ayar ekranı ve paketleme yoktur.

English version: [README.md](README.md)

## Lisans

Apache-2.0, bkz. [LICENSE](LICENSE) ve [NOTICE](NOTICE). Rust paketleri ve Tauri simge şablonu kendi lisanslarını korur (bağımlılık listesi için `Cargo.lock`).
