# onparmak

```
┌─┐┌┐┌┌─┐┌─┐┬─┐┌┬┐┌─┐┬┌─
│ ││││├─┘├─┤├┬┘│││├─┤├┴┐
└─┘┘└┘┴  ┴ ┴┴└─┴ ┴┴ ┴┴ ┴
```

**on parmak, tek ritim.** Terminalde çalışan, zarif bir Türkçe on parmak
klavye eğitmeni. Türkçe **F** ve **Q** düzenlerini bilir, tuşları parmak
parmak öğretir, sonra gerçek Türkçe metinlerle hızlandırır.

Rust + [ratatui](https://ratatui.rs) ile yazıldı; tek dosyalık, bağımlılıksız
bir ikili olarak Linux, macOS ve Windows'ta çalışır.

## Neler var

- **16 derslik müfredat (TypingClub tarzı).** İşaret parmaklarından başlar;
  orta, yüzük, serçe, uzanışlar, üst sıra, alt sıra, büyük harf, rakamlar,
  noktalama ve atasözleriyle biter. Her ders üç adım: *Tanışma* (yeni tuşların
  tekrarı), *Karışık* (açık harflerden kelimeler), *Kelimeler* (yalnızca o ana
  kadar öğrendiğin harflerden oluşan **gerçek Türkçe kelimeler**).
- **Derslerde yanlış tuş ilerlemez.** İmleç doğrusunu bekler, yanlış bastığın
  tuş klavyede kırmızı yanar.
- **Canlı ekran klavyesi.** Sıradaki tuş, basman gereken parmağın rengiyle
  yanar; altında hangi parmak olduğu ve gerekiyorsa hangi Shift'e basacağın
  yazar.
- **Yıldızlar.** %90 doğruluk bir yıldız, %94 ve 15 wpm iki, %97 ve 25 wpm üç
  yıldız. Bir dersi bitirmek sonrakinin kilidini açar.
- **8 kelime seti** (her biri en az 500 kelime): en sık 500, sık 1000, gerçekçi
  metin (büyük harf + noktalama), çekimli kelimeler, uzun kelimeler, Türkçe
  karakterler, F klavyede zor kelimeler, atasözleri. Ayrıntılar:
  [data/README.md](data/README.md).
- **Süreli ve kelime sayılı modlar:** 15/30/60/120 saniye, 25/50/100 kelime.
- **Hayalet imleç.** Aynı moddaki rekor hızın metinde altı çizili bir hayalet
  olarak seninle yarışır.
- **Zayıf tuşlar modu.** Her tuşun hata oranını ve gecikmesini tutar, en çok
  zorlandığın harfleri içeren kelimelerle özel alıştırma üretir.
- **Sonuç ekranı.** Kocaman wpm, doğruluk, ham hız, istikrar, saniye saniye
  hız grafiği (hataların işaretli), en çok hata yapılan ve en yavaş tuşlar.
- **İstatistikler.** Gidişat grafiği, rekorlar ve klavye üzerinde **tuş ısı
  haritası**.
- **Günlük seri** ve ana ekranda **günün atasözü**.
- **Temalar.** Orman, Gece, Kehribar. [Omarchy](https://omarchy.org) kuruluysa
  aktif sistem temasını otomatik okur ve listeye ekler.

## Kurulum

### Hazır ikili (önerilen)

[Releases](../../releases) sayfasından sistemine uygun arşivi indir ya da:

```sh
curl -fsSL https://raw.githubusercontent.com/KULLANICI_ADI/onparmak/main/install.sh | bash
```

Betik ikiliyi `~/.local/bin/onparmak` olarak kurar ve Linux'ta uygulama
başlatıcısına ekler.

### Kaynaktan

```sh
cargo install --git https://github.com/KULLANICI_ADI/onparmak
```

## Kullanım

```sh
onparmak
```

| Tuş | İş |
|---|---|
| `↑ ↓` / `j k` | listede gezin |
| `← →` / `h l` | adım, mod ya da ayar değeri seç |
| `enter` | başla / devam |
| `tab` | aynı alıştırmayı yeniden başlat |
| `esc` | geri |
| `ctrl+w`, `ctrl+backspace` | son kelimeyi sil |
| `?` | yardım |
| `ctrl+c` | çık |

İlerleme `~/.local/share/onparmak/state.json` dosyasında tutulur (macOS'ta
`~/Library/Application Support/onparmak`, Windows'ta `%APPDATA%\onparmak`).
`ONPARMAK_STATE` ortam değişkeniyle başka bir dosya gösterebilirsin.

## Ölçüler

- **wpm**: doğru karakter sayısı ÷ 5, dakika başına.
- **ham**: hatalı karakterler dahil hız.
- **doğruluk**: basılan tuşların doğru olan yüzdesi; sonradan düzeltilen
  hatalar da sayılır.
- **istikrar**: saniyelik hızın ne kadar dengeli olduğu (100 = metronom).

## Geliştirme

Her push'ta GitHub Actions Linux, macOS ve Windows'ta derler ve testleri
çalıştırır; Linux ikilisini artifact olarak yükler. `v*` etiketi
(`git tag v0.1.0 && git push --tags`) beş platform için sürüm arşivlerini
üretip bir GitHub Release yayınlar.

```
src/
  main.rs      giriş noktası
  app.rs       ekranlar, gezinme, tuş işleme
  ui.rs        tüm çizimler, ekran klavyesi, grafikler
  session.rs   yazma motoru: wpm, doğruluk, örnekler, tuş istatistikleri
  lessons.rs   ders planı ve alıştırma üretimi
  layout.rs    Türkçe F ve Q düzenleri, parmak eşlemesi
  data.rs      gömülü kelime setleri, pratik metni üretimi
  state.rs     kalıcı ilerleme
  theme.rs     temalar ve Omarchy entegrasyonu
  text.rs      Türkçe büyük/küçük harf, metin kaydırma
data/sets/     kelime setleri
```

## Lisans

Kod MIT. Kelime setleri CC BY-SA 4.0 (bkz. [data/README.md](data/README.md)).
