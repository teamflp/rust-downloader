import { useState, useEffect } from 'react';
import { BarChart, Bar, LineChart, Line, PieChart, Pie, Cell, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, AreaChart, Area, ComposedChart } from 'recharts';
import { BarChart3, TrendingUp, PieChart as PieChartIcon, HardDrive, RefreshCw } from 'lucide-react';
import { downloadAPI } from '../api/client';
import './StatisticsDashboard.css';

const StatisticsDashboard = () => {
  const [statistics, setStatistics] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  const fetchStatistics = async () => {
    try {
      setLoading(true);
      setError(null);
      const stats = await downloadAPI.getStatistics();
      setStatistics(stats);
    } catch (err) {
      console.error('Failed to fetch statistics:', err);
      const errorMessage = err.response?.data?.message || err.message || 'Erreur lors du chargement des statistiques';
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStatistics();
  }, []);

  const formatBytes = (bytes) => {
    if (!bytes || bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatDate = (dateString) => {
    const date = new Date(dateString);
    return date.toLocaleDateString('fr-FR', { day: '2-digit', month: 'short' });
  };

  if (loading) {
    return (
      <div className="statistics-dashboard">
        <div className="statistics-loading">
          <RefreshCw size={32} className="animate-spin" />
          <p>Chargement des statistiques...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="statistics-dashboard">
        <div className="statistics-error">
          <p>{error}</p>
          <button onClick={fetchStatistics} className="btn-primary">
            Réessayer
          </button>
        </div>
      </div>
    );
  }

  if (!statistics) {
    return (
      <div className="statistics-dashboard">
        <div className="statistics-empty">
          <p>Aucune statistique disponible</p>
        </div>
      </div>
    );
  }

  // Prepare data for charts
  const trendsData = statistics.trends.map(t => ({
    date: formatDate(t.date),
    count: t.count,
    size: t.total_size,
  }));

  const typeColors = {
    video: '#ef4444',
    audio: '#3b82f6',
    instrumental: '#8b5cf6',
  };

  const statusColors = {
    completed: '#10b981',
    failed: '#ef4444',
    downloading: '#3b82f6',
    pending: '#f59e0b',
    processing: '#06b6d4',
    converting: '#8b5cf6',
  };

  const spaceEvolutionData = statistics.space_evolution.map(s => ({
    date: formatDate(s.date),
    size: s.total_size,
    count: s.file_count,
  }));

  return (
    <div className="statistics-dashboard">
      <div className="statistics-header">
        <h2>
          <BarChart3 size={24} />
          Statistiques Détaillées
        </h2>
        <button onClick={fetchStatistics} className="btn-icon" title="Actualiser" disabled={loading}>
          <RefreshCw size={18} className={loading ? 'animate-spin' : ''} />
        </button>
      </div>

      {/* Summary Cards */}
      <div className="statistics-summary">
        <div className="summary-card">
          <div className="summary-icon">
            <BarChart3 size={24} />
          </div>
          <div className="summary-content">
            <h3>{statistics.total_downloads}</h3>
            <p>Total téléchargements</p>
          </div>
        </div>
        <div className="summary-card">
          <div className="summary-icon success">
            <TrendingUp size={24} />
          </div>
          <div className="summary-content">
            <h3>{statistics.success_rate.toFixed(1)}%</h3>
            <p>Taux de succès</p>
          </div>
        </div>
        <div className="summary-card">
          <div className="summary-icon">
            <HardDrive size={24} />
          </div>
          <div className="summary-content">
            <h3>{formatBytes(statistics.total_size)}</h3>
            <p>Espace total utilisé</p>
          </div>
        </div>
        <div className="summary-card">
          <div className="summary-icon">
            <PieChartIcon size={24} />
          </div>
          <div className="summary-content">
            <h3>{formatBytes(statistics.average_file_size)}</h3>
            <p>Taille moyenne</p>
          </div>
        </div>
      </div>

      {/* Charts Grid */}
      <div className="statistics-charts">
        {/* Download Trends */}
        <div className="chart-container">
          <h3>Tendances des téléchargements (30 derniers jours)</h3>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={trendsData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="date" angle={-45} textAnchor="end" height={80} />
              <YAxis yAxisId="left" />
              <YAxis yAxisId="right" orientation="right" />
              <Tooltip 
                formatter={(value, name) => {
                  if (name === 'count') return [value, 'Nombre'];
                  if (name === 'size') return [formatBytes(value), 'Taille'];
                  return value;
                }}
              />
              <Legend />
              <Bar yAxisId="left" dataKey="count" fill="#3b82f6" name="Nombre de téléchargements" />
            </BarChart>
          </ResponsiveContainer>
        </div>

        {/* Type Distribution */}
        <div className="chart-container">
          <h3>Répartition par type</h3>
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={statistics.type_distribution}
                cx="50%"
                cy="50%"
                labelLine={false}
                label={({ name, percentage }) => `${name}: ${percentage.toFixed(1)}%`}
                outerRadius={100}
                fill="#8884d8"
                dataKey="count"
              >
                {statistics.type_distribution.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={typeColors[entry.download_type] || '#8884d8'} />
                ))}
              </Pie>
              <Tooltip 
                formatter={(value, name, props) => {
                  if (name === 'count') return [value, 'Nombre'];
                  return [`${formatBytes(props.payload.total_size)}`, 'Taille totale'];
                }}
              />
              <Legend />
            </PieChart>
          </ResponsiveContainer>
        </div>

        {/* Status Distribution */}
        <div className="chart-container">
          <h3>Répartition par statut</h3>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={statistics.status_distribution}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="status" />
              <YAxis />
              <Tooltip 
                formatter={(value, name) => {
                  if (name === 'count') return [value, 'Nombre'];
                  if (name === 'percentage') return [`${value.toFixed(1)}%`, 'Pourcentage'];
                  return value;
                }}
              />
              <Legend />
              <Bar dataKey="count" fill="#8884d8">
                {statistics.status_distribution.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={statusColors[entry.status] || '#8884d8'} />
                ))}
              </Bar>
            </BarChart>
          </ResponsiveContainer>
        </div>

        {/* Space Evolution */}
        <div className="chart-container full-width">
          <h3>Évolution de l'espace utilisé (30 derniers jours)</h3>
          <ResponsiveContainer width="100%" height={300}>
            <ComposedChart data={spaceEvolutionData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="date" angle={-45} textAnchor="end" height={80} />
              <YAxis yAxisId="left" />
              <YAxis yAxisId="right" orientation="right" />
              <Tooltip 
                formatter={(value, name) => {
                  if (name === 'size') return [formatBytes(value), 'Taille'];
                  if (name === 'count') return [value, 'Nombre de fichiers'];
                  return value;
                }}
              />
              <Legend />
              <Area yAxisId="left" type="monotone" dataKey="size" stroke="#10b981" fill="#10b981" fillOpacity={0.6} name="Espace utilisé" />
              <Line yAxisId="right" type="monotone" dataKey="count" stroke="#3b82f6" strokeWidth={2} name="Nombre de fichiers" />
            </ComposedChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  );
};

export default StatisticsDashboard;

