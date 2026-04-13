import Foundation
import CoreLocation
import WeatherKit

enum SceneWeather: String {
    case sunny
    case cloudy
    case rainy
    case snowy
    case stormy
    case night
}

class WeatherService: NSObject, CLLocationManagerDelegate {
    weak var model: AppModel?
    private let locationManager = CLLocationManager()
    private var lastUpdate: Date?
    private var timer: Timer?

    override init() {
        super.init()
        locationManager.delegate = self
        locationManager.desiredAccuracy = kCLLocationAccuracyKilometer // city-level only
    }

    func startMonitoring() {
        locationManager.requestWhenInUseAuthorization()
        locationManager.startUpdatingLocation()

        // Refresh weather every 15 minutes
        timer = Timer.scheduledTimer(withTimeInterval: 900, repeats: true) { [weak self] _ in
            self?.fetchWeather()
        }
    }

    func stopMonitoring() {
        locationManager.stopUpdatingLocation()
        timer?.invalidate()
    }

    func locationManager(_ manager: CLLocationManager, didUpdateLocations locations: [CLLocation]) {
        guard let location = locations.last else { return }

        // Only fetch if we haven't recently
        if let last = lastUpdate, Date().timeIntervalSince(last) < 600 { return }
        lastUpdate = Date()

        fetchWeather(at: location)
    }

    func locationManager(_ manager: CLLocationManager, didFailWithError error: Error) {
        eprintln("[DagashiIsland] Location error: \(error.localizedDescription)")
        // Default to sunny
        DispatchQueue.main.async {
            self.model?.sceneWeather = .sunny
        }
    }

    func locationManagerDidChangeAuthorization(_ manager: CLLocationManager) {
        switch manager.authorizationStatus {
        case .authorizedAlways, .authorized:
            manager.startUpdatingLocation()
        case .denied, .restricted:
            // No permission — stay sunny
            DispatchQueue.main.async {
                self.model?.sceneWeather = .sunny
            }
        default:
            break
        }
    }

    private func fetchWeather(at location: CLLocation? = nil) {
        guard let loc = location ?? locationManager.location else { return }

        Task {
            do {
                let weather = try await WeatherService.shared.weather(for: loc)
                let condition = weather.currentWeather.condition
                let isDay = weather.currentWeather.isDaylight

                let scene: SceneWeather
                if !isDay {
                    scene = .night
                } else {
                    switch condition {
                    case .clear, .hot, .mostlyClear:
                        scene = .sunny
                    case .cloudy, .mostlyCloudy, .partlyCloudy, .haze, .foggy, .smoky:
                        scene = .cloudy
                    case .rain, .heavyRain, .drizzle, .sunShowers:
                        scene = .rainy
                    case .snow, .heavySnow, .flurries, .sleet, .freezingRain, .freezingDrizzle, .blizzard:
                        scene = .snowy
                    case .thunderstorms, .strongStorms, .tropicalStorm, .hurricane:
                        scene = .stormy
                    default:
                        scene = .sunny
                    }
                }

                await MainActor.run {
                    self.model?.sceneWeather = scene
                    eprintln("[DagashiIsland] Weather: \(condition.description) → \(scene.rawValue)")
                }
            } catch {
                eprintln("[DagashiIsland] Weather fetch failed: \(error)")
            }
        }
    }
}

// WeatherKit's WeatherService conflicts with our class name
private extension WeatherService {
    static let shared = WeatherKit.WeatherService.shared
}

private func eprintln(_ msg: String) {
    fputs(msg + "\n", stderr)
}
