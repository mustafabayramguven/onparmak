# Kelime setleri

| Dosya | İçerik | Kelime |
|---|---|---|
| `en-sik-500.txt` | Türkçede en sık 500 kelime | 500 |
| `sik-1000.txt` | Sonraki 1000 sık kelime | 1000 |
| `cekimli-kelimeler.txt` | Ekleriyle sık geçen kelimeler (biliyorum, olduğunu…) | 700 |
| `uzun-kelimeler.txt` | 10 harf ve üzeri sık kelimeler | 600 |
| `turkce-karakterler.txt` | En az iki tane ç, ğ, ı, ö, ş, ü içerenler | 600 |
| `zor-kelimeler.txt` | F klavyede aynı parmak / sıra atlaması zorluğu en yüksek olanlar | 600 |
| `atasozleri.txt` | 159 atasözü, büyük harf ve noktalamayla | 704 |

Uygulama bunlara ek olarak **Gerçekçi metin** setini çalışırken üretir: sık
kelimelerden büyük harf, virgül, nokta, soru ve ünlem içeren cümleler.

## Kaynak ve lisans

İlk altı set, [hermitdave/FrequencyWords](https://github.com/hermitdave/FrequencyWords)
deposundaki OpenSubtitles 2018 Türkçe sıklık listesinden (`tr_50k.txt`) türetilmiştir.
O veri **CC BY-SA 4.0** lisanslıdır; bu setler de aynı lisansla dağıtılır.

Türetme adımları:

1. Yalnızca Türkçe alfabedeki harflerden oluşan kelimeler alındı (q, w, x ve
   yabancı karakterler elendi).
2. Sık kelime setleri TDK yazım kılavuzu kelime listesiyle süzüldü; böylece
   altyazılardaki isimler ve İngilizce kelimeler çıkarıldı. Çekimli, uzun ve
   zor kelimelerde kökün sözlükte bulunması şartı arandı. (TDK listesi yalnızca
   filtre olarak kullanıldı, depoda yer almaz.)
3. Küfür ve uygunsuz kelimeler bir engel listesiyle temizlendi.
4. Zorluk puanı: F düzeninde art arda aynı parmağın kullanılması (+3),
   sıra atlaması (satır farkı × 1,2) ve ana sıra dışı tuşlar (+0,6),
   harf başına ortalama.

`atasozleri.txt` halk kültürüne ait atasözlerinden derlenmiştir (kamu malı).

Her dosyanın ilk iki `# ` satırı başlık ve açıklamadır; uygulama bunları
listede gösterir. Yeni bir set eklemek için aynı biçimde bir dosya yazıp
`src/data.rs` içindeki `RAW` listesine eklemek yeterli.
