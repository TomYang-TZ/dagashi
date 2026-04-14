import Foundation

enum SceneWeather: String {
    case sunny
    case cloudy
    case rainy
    case snowy
    case stormy
    case night
}

class WeatherService {
    weak var model: AppModel?
    private var timer: Timer?
    private var cachedLat: Double?
    private var cachedLon: Double?

    func startMonitoring() {
        // Fetch immediately, then every 30 minutes
        fetchWeather()
        timer = Timer.scheduledTimer(withTimeInterval: 1800, repeats: true) { [weak self] _ in
            self?.fetchWeather()
        }
    }

    func stopMonitoring() {
        timer?.invalidate()
    }

    private func fetchWeather() {
        // Step 1: get coordinates (cached after first call)
        if let lat = cachedLat, let lon = cachedLon {
            fetchFromOpenMeteo(lat: lat, lon: lon)
        } else {
            geolocate { [weak self] lat, lon in
                self?.cachedLat = lat
                self?.cachedLon = lon
                self?.fetchFromOpenMeteo(lat: lat, lon: lon)
            }
        }
    }

    /// IP-based geolocation to get coordinates for Open-Meteo.
    private func geolocate(completion: @escaping (Double, Double) -> Void) {
        guard let url = URL(string: "http://ip-api.com/json/?fields=lat,lon") else { return }

        var request = URLRequest(url: url)
        request.timeoutInterval = 10

        URLSession.shared.dataTask(with: request) { data, _, error in
            guard let data = data, error == nil,
                  let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                  let lat = json["lat"] as? Double,
                  let lon = json["lon"] as? Double else {
                fputs("[DagashiIsland] Geolocation failed: \(error?.localizedDescription ?? "parse error")\n", stderr)
                return
            }
            fputs("[DagashiIsland] Location: \(lat), \(lon)\n", stderr)
            completion(lat, lon)
        }.resume()
    }

    /// Fetch current weather from Open-Meteo (free, no API key, 15-min updates).
    private func fetchFromOpenMeteo(lat: Double, lon: Double) {
        let urlStr = "https://api.open-meteo.com/v1/forecast?latitude=\(lat)&longitude=\(lon)&current=weather_code,is_day&timezone=auto"
        guard let url = URL(string: urlStr) else { return }

        var request = URLRequest(url: url)
        request.timeoutInterval = 10

        URLSession.shared.dataTask(with: request) { [weak self] data, _, error in
            guard let self = self, let data = data, error == nil else {
                fputs("[DagashiIsland] Open-Meteo fetch failed: \(error?.localizedDescription ?? "no data")\n", stderr)
                return
            }

            guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                  let current = json["current"] as? [String: Any],
                  let code = current["weather_code"] as? Int,
                  let isDay = current["is_day"] as? Int else {
                fputs("[DagashiIsland] Open-Meteo parse failed\n", stderr)
                return
            }

            let scene = self.mapWMOCode(code, isNight: isDay == 0)

            DispatchQueue.main.async {
                self.model?.sceneWeather = scene
                fputs("[DagashiIsland] Weather WMO \(code) (isDay=\(isDay)) → \(scene.rawValue)\n", stderr)
            }
        }.resume()
    }

    /// Map WMO weather interpretation codes to scene weather.
    /// https://open-meteo.com/en/docs#weathervariables
    private func mapWMOCode(_ code: Int, isNight: Bool) -> SceneWeather {
        if isNight { return .night }

        switch code {
        case 0, 1:         // Clear sky, Mainly clear
            return .sunny
        case 2, 3:         // Partly cloudy, Overcast
            return .cloudy
        case 45, 48:       // Fog, Depositing rime fog
            return .cloudy
        case 51, 53, 55:   // Drizzle: light, moderate, dense
            return .rainy
        case 56, 57:       // Freezing drizzle: light, dense
            return .rainy
        case 61, 63, 65:   // Rain: slight, moderate, heavy
            return .rainy
        case 66, 67:       // Freezing rain: light, heavy
            return .rainy
        case 71, 73, 75:   // Snow fall: slight, moderate, heavy
            return .snowy
        case 77:           // Snow grains
            return .snowy
        case 80, 81, 82:   // Rain showers: slight, moderate, violent
            return .rainy
        case 85, 86:       // Snow showers: slight, heavy
            return .snowy
        case 95:           // Thunderstorm: slight or moderate
            return .stormy
        case 96, 99:       // Thunderstorm with hail: slight, heavy
            return .stormy
        default:
            return .sunny
        }
    }
}
