# Horizon Başlatıcı — Değişiklikler

Bu belge, `horizon/launcher` üzerinde yapılan değişiklikleri ve nasıl kullanılacağını anlatır.

## 1. Bilgisayarın donması

**Neden:** Başlatıcı, açıkken arka plandaki gün batımı sahnesini sürekli yeniden çiziyordu.
Her karede 1920×1080'lik pencerenin tamamı (cam plakalar, bulanıklık, yıldızlar) yeniden
çizildiği için 4 çekirdekli / 3,5 GB RAM'li bir dizüstünde bu, %25–30 CPU'ya ve düşük kare
hızına yol açıyordu. Ölçüm (Intel N100):

| Durum | Boştaki CPU |
|---|---|
| Animasyon açık (eski hâl) | %24–27 |
| Animasyon kapalı | %1,4–2,7 |

**Yapılanlar:**

- `scenery/src/scene.rs`: sahne artık fare paralaksı oturduktan sonra her 31 ms'de değil,
  92 ms'de bir (Performans modunda 160 ms) yeniden çiziliyor. Fare hareket ederken akıcılık
  korunuyor, boştayken yeniden çizim ~5 kat azalıyor.
- `src/launcher_setup.rs`: 4 veya daha az iş parçacığı olan bilgisayarlarda launcher ilk açılışta
  **Performans modunu** kendiliğinden açar (Ayarlar > Görünüm'den kapatılabilir). Bu mod cam
  plakaları düz renge çevirir, bulanıklığı kaldırır: GPU yükü belirgin düşer.
- Performans modunda açılış animasyonu atlanır (zaten hızlı açılış isteyen makineler için).
- İstersen tamamen durdurmak için: `HORIZON_LAUNCHER_MOTION=off ./horizon-launcher` ya da
  Ayarlar > Görünüm > Hareket = Kapalı.

## 2. Arayüz sadeleştirmesi

- Ana sayfadaki "N kişi şu anda oynuyor · resmi SMP'de M" bilgisi tek satırlık gürültü yerine
  başlığın altında ikinci, daha sakin bir satıra taşındı; artık cümle tek parça okunuyor.
- Çevrimdışı profiller için ayrı, sade bir kart eklendi (Ayarlar > Hesaplar).
- Üst çubuktaki hesap rozeti çevrimdışı profilin adını da gösteriyor.
- Türkçe çeviriler tamamlandı: arayüzde kalan ~195 İngilizce metin Türkçeye çevrildi
  (önceden ekranlar yarı İngilizce yarı Türkçeydi).

## 3. Çevrimdışı (offline) giriş

Microsoft hesabı olmadan oynama. Oyun, adı `--name` ile alır ve Minecraft'ın kendi çevrimdışı
UUID'sini (`OfflinePlayer:<ad>`) türetir.

**Arayüzden:** Ayarlar > Hesaplar > **Çevrimdışı profiller** → ad yaz → **Ekle**.
Listeden bir profile tıklamak onu "oynayan" yapar; **Microsoft hesabını kullan** ile geri
dönülür. Üst çubuktaki hesap menüsünde de görünürler.

**Komut satırından:**

```bash
horizon-launcher account offline              # liste (* = oynayan)
horizon-launcher account offline add Steve    # ekle ve oynayan yap
horizon-launcher account offline select Alex
horizon-launcher account offline remove Alex
horizon-launcher account offline microsoft    # yeniden Microsoft hesabıyla oyna
```

**Ne çalışır:** Tek oyunculu, LAN dünyaları ve `online-mode=false` sunucular.
**Ne çalışmaz:** `online-mode=true` sunucular — bu sunucunun kuralıdır, başlatıcının değil.

Profiller `offline.json` dosyasında tutulur (launcher veri klasörü). Microsoft hesapları
silinmez; ikisi yan yana durur.

## 4. Doğrulama

- `cargo test -p horizon-launcher --lib offline::` → 4 test geçti (ad kuralları, seçim,
  silme, kaydetme/yükleme).
- `cargo test -p horizon-launcher --lib i18n::` → geçti (çeviri delikleri/anahtarları tutarlı).
- Arayüz gerçek X11 oturumunda açılıp ekran görüntüleriyle doğrulandı: ana sayfa, Ayarlar >
  Hesaplar, çevrimdışı kart, profil ekleme.

## 5. Derleme ve çalıştırma

```bash
cd launcher
export RUST_FONTCONFIG_DLOPEN=1     # fontconfig geliştirme başlığı olmayan sistemlerde
cargo build --release
./target/release/horizon-launcher
```

Not: Bu makinede `libxkbcommon`, `libxcb` geliştirme paketleri yok; bağlama için
`target/debug/build/libmimalloc-sys/*/out` klasörüne `.so` bağlantıları konuldu.
Kalıcı çözüm: `sudo apt install libxkbcommon-dev libxkbcommon-x11-dev libxcb1-dev libfontconfig1-dev`.

## Değişen dosyalar

```
launcher/src/offline.rs                 (yeni) çevrimdışı profil modeli + testler
launcher/src/lib.rs                     modül kaydı
launcher/src/account.rs                 GameSession.name + as_offline()
launcher/src/launcher_play.rs           Play/başlatma çevrimdışı yolu
launcher/src/backend/developer_play.rs  CLI play çevrimdışı yolu
launcher/src/developer_play.rs          pencere Play --name geçişi
launcher/src/glass_account.rs           Ayarlar'daki çevrimdışı kart + "kim oynuyor"
launcher/src/blocks_settings.rs         eski görünümde çevrimdışı satırları
launcher/src/frame.rs                   hesap menüsünde profiller, rozet adı
launcher/src/switch_page.rs             play_as_offline, who_plays
launcher/src/owner_page.rs              sahiplik kapısı çevrimdışını engellemiyor
launcher/src/glass_home.rs              Play etiketi, sade sayı satırı
launcher/src/home.rs, glass_home_lists.rs  play_owner()
launcher/src/launcher_setup.rs          performans modu varsayılanı, açılış animasyonu
launcher/src/glass_account.rs, main.rs, launcher_setup.rs  offline alanları
launcher/src/cli.rs                     account offline alt komutları
launcher/scenery/src/scene.rs           boştaki yeniden çizim aralığı
launcher/lang/tr.json                   Türkçe çeviriler
launcher/README.md                      belge
```
