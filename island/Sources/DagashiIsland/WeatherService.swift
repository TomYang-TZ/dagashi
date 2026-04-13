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

    func startMonitoring() {
        // Fetch immediately, then every 15 minutes
        fetchWeather()
        timer = Timer.scheduledTimer(withTimeInterval: 900, repeats: true) { [weak self] _ in
            self?.fetchWeather()
        }
    }

    func stopMonitoring() {
        timer?.invalidate()
    }

    private func fetchWeather() {
        // wttr.in — free, no API key, uses IP geolocation
        guard let url = URL(string: "https://wttr.in/?format=j1") else { return }

        var request = URLRequest(url: url)
        request.timeoutInterval = 10

        URLSession.shared.dataTask(with: request) { [weak self] data, _, error in
            guard let self = self, let data = data, error == nil else {
                fputs("[DagashiIsland] Weather fetch failed: \(error?.localizedDescription ?? "no data")\n", stderr)
                return
            }

            guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                  let conditions = json["current_condition"] as? [[String: Any]],
                  let current = conditions.first,
                  let codeStr = current["weatherCode"] as? String,
                  let code = Int(codeStr) else {
                return
            }

            // Check if it's night (astronomy data)
            var isNight = false
            if let astronomy = (json["weather"] as? [[String: Any]])?.first?["astronomy"] as? [[String: Any]],
               let astro = astronomy.first,
               let sunset = astro["sunset"] as? String,
               let sunrise = astro["sunrise"] as? String {
                let now = Date()
                let fmt = DateFormatter()
                fmt.dateFormat = "hh:mm a"
                if let sunsetTime = fmt.date(from: sunset),
                   let sunriseTime = fmt.date(from: sunrise) {
                    let cal = Calendar.current
                    let nowMinutes = cal.component(.hour, from: now) * 60 + cal.component(.minute, from: now)
                    let sunsetMinutes = cal.component(.hour, from: sunsetTime) * 60 + cal.component(.minute, from: sunsetTime)
                    let sunriseMinutes = cal.component(.hour, from: sunriseTime) * 60 + cal.component(.minute, from: sunriseTime)
                    isNight = nowMinutes > sunsetMinutes || nowMinutes < sunriseMinutes
                }
            }

            let scene = self.mapWeatherCode(code, isNight: isNight)

            DispatchQueue.main.async {
                self.model?.sceneWeather = scene
                fputs("[DagashiIsland] Weather code \(code) → \(scene.rawValue)\n", stderr)
            }
        }.resume()
    }

    // wttr.in weather codes (WWO codes)
    // https://www.worldweatheronline.com/developer/api/docs/weather-icons.aspx
    private func mapWeatherCode(_ code: Int, isNight: Bool) -> SceneWeather {
        if isNight { return .night }

        switch code {
        case 113: // Clear/Sunny
            return .sunny
        case 116, 119, 122: // Partly cloudy, Cloudy, Overcast
            return .cloudy
        case 143, 248, 260: // Mist, Fog, Freezing fog
            return .cloudy
        case 176, 263, 266, 293, 296, 299, 302, 305, 308, 311, 314, 353, 356, 359:
            // Various rain types
            return .rainy
        case 179, 182, 185, 227, 230, 317, 320, 323, 326, 329, 332, 335, 338, 350, 362, 365, 368, 371, 374, 377:
            // Various snow/sleet types
            return .snowy
        case 200, 386, 389, 392, 395:
            // Thunder
            return .stormy
        default:
            return .sunny
        }
    }
}
